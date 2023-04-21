use {
    proc_macro2::TokenStream,
    quote::ToTokens,
    std::hash::{Hash, Hasher},
};

pub struct TokensSetItem<'input> {
    string: String,
    tokens: &'input TokenStream,
}

impl<'input> From<&'input TokenStream> for TokensSetItem<'input> {
    fn from(tokens: &'input TokenStream) -> Self {
        TokensSetItem {
            string: tokens.to_string(),
            tokens,
        }
    }
}

impl PartialEq for TokensSetItem<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.string.eq(&other.string)
    }
}

impl Eq for TokensSetItem<'_> {}

impl Hash for TokensSetItem<'_> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.string.hash(state)
    }
}

impl ToTokens for TokensSetItem<'_> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        self.tokens.to_tokens(stream)
    }
}
