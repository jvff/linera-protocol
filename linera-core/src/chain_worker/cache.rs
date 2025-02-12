// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! A cache of loaded chain workers.

pub struct ChainWorkerCache<StorageClient> {
    /// Access to local persistent storage.
    storage: StorageClient,
    /// Configuration options for the [`ChainWorkerState`]s.
    config: ChainWorkerConfig,
    /// Cached hashed certificate values by hash.
    recent_hashed_certificate_values: Arc<ValueCache<CryptoHash, HashedCertificateValue>>,
    /// Cached blobs by `BlobId`.
    recent_blobs: Arc<ValueCache<BlobId, Blob>>,
    /// Chain IDs that should be tracked by a worker.
    tracked_chains: Option<Arc<RwLock<HashSet<ChainId>>>>,
    /// The [`ChainWorkerState`]s that are active in cache.
    active_workers: SyncMutex<LruCache<ChainId, ChainWorkerReference<StorageClient>>>,
    /// The notifiers for [`ChainWorkerState`]s that are being loaded into the cache.
    loading_workers: SyncMutex<HashMap<ChainId, Arc<Notify>>>,
}

impl<StorageClient> ChainWorkerCache<StorageClient> {
    pub fn new(
        size: NonZeroUsize,
        storage_client: StorageClient,
        config: ChainWorkerConfig,
        recent_hashed_certificate_values: Arc<ValueCache<CryptoHash, HashedCertificateValue>>,
        recent_blobs: Arc<ValueCache<BlobId, Blob>>,
        tracked_chains: Option<Arc<RwLock<HashSet<ChainId>>>>,
    ) -> Self {
        ChainWorkerCache {
            storage_client,
            config,
            recent_hashed_certificate_values,
            recent_blobs,
            tracked_chains,
            active_workers: SyncMutex::new(LruCache::new(size)),
            loading_workers: SyncMutex::new(HashMap::new()),
        }
    }

    pub async fn get(
        &self,
        chain_id: ChainId,
    ) -> Result<ChainWorkerEndpoint<StorageClient>, WorkerError> {
        let chain_worker = loop {
            match self.try_get(chain_id) {
                Reference(worker) => break worker,
                Missing => break self.load_new_worker(chain_id).await?,
                Loading(notifier) => {
                    notifier.notified().await;
                    notifier.notify_one();
                }
            }
        };

        Ok(chain_worker.lock_owned().await)
    }

    fn try_get(&self, chain_id: ChainId) -> Result<TryGetOutcome<StorageClient>, ExecutionError> {
        let mut active_workers = self.active_workers.lock().unwrap();

        if let Some(worker) = active_workers.get(chain_id) {
            Ok(TryGetOutcome::Reference(worker.clone()))
        } else {
            let mut loading_workers = self.loading_workers.lock().unwrap();

            if let Some(notifier) = loading_workers.get(chain_id) {
                Ok(TryGetOutcome::Loading(notifier.clone()))
            } else {
                let notifier = Arc::new(Notify::new());

                loading_workers.push(chain_id, notifier.clone());

                Ok(TryGetOutcome::Missing(notifier))
            }
        }
    }

    async fn load_new_worker(
        &self,
        chain_id: ChainId,
    ) -> Result<ChainWorkerReference<StorageClient>, WorkerError> {
        self.ensure_space_for_new_worker().await?;

        let (service_runtime_thread, execution_state_receiver, runtime_request_sender) =
            Self::spawn_service_runtime_actor(chain_id);

        let worker = ChainWorkerState::load(
            self.config.clone(),
            self.storage.clone(),
            self.certificate_value_cache.clone(),
            self.blob_cache.clone(),
            self.tracked_chains.clone(),
            chain_id,
            execution_state_receiver,
            runtime_request_sender,
        )
        .await?;

        Ok(self.finish_loading_worker(worker, chain_id))
    }

    async fn ensure_space_for_new_worker(&self) {
        let retry_delay = Duration::from_millis(250);

        timeout(Duration::from_secs(3), async {
            while self.try_to_make_space_for_new_worker() {
                warn!(
                    "No chain worker candidates found for eviction, retrying in {retry_delay:?}..."
                );
                sleep(retry_delay).await;
            }
        })
        .await
        .map_err(|_| WorkerError::FullChainWorkerCache)
    }

    fn try_to_make_space_for_new_worker(&self) -> bool {
        let mut active_workers = self.active_workers.lock().unwrap();
        let loading_workers_count = self.loading_workers.lock().unwrap().len();

        while active_workers.len() + loading_workers_count >= usize::from(active_workers.cap()) {
            let Some((&chain_to_evict, _)) = active_workers
                .iter()
                .rev()
                .find(|(_, reference)| reference.strong_count <= 1)
            else {
                return false;
            };

            active_workers.pop(&chain_to_evict);
        }

        true
    }

    fn finish_loading_worker(
        &self,
        worker: ChainWorkerState<StorageClient>,
        chain_id: ChainId,
    ) -> ChainWorkerReference<StorageClient> {
        let reference = Arc::new(worker);

        let mut active_workers = self.active_workers.lock().unwrap();
        active_workers.insert(chain_id, reference.clone());

        let mut loading_workers = self.loading_workers.lock().unwrap();
        let mut notifier = loading_workers
            .remove(chain_id)
            .expect("All chain workers being initialized should have a notifier");
        notifier.notify_one();

        Ok(reference)
    }
}

enum TryGetOutcome<StorageClient> {
    Reference(ChainWorkerReference<StorageClient>),
    Loading(Arc<Notify>),
    Missing(Arc<Notify>),
}
