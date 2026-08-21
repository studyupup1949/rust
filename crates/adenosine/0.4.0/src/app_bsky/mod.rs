/// app.bsky types (manually entered)
use serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct Subject {
    pub uri: String,
    // TODO: CID is required
    pub cid: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct StrongRef {
    pub uri: String,
    pub cid: String,
}

/// Generic over Re-post and Like
#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct RefRecord {
    pub subject: Subject,
    pub createdAt: String,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct FollowSubject {
    pub did: String,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct FollowRecord {
    pub subject: FollowSubject,
    pub createdAt: String,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct ProfileRecord {
    pub displayName: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<Blob>,
    pub banner: Option<Blob>,
    // TODO: self-labels
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct Label {
    pub src: String,
    pub uri: String,
    pub cid: Option<String>,
    pub val: String,
    pub neg: Option<bool>,
    pub cts: String,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct ProfileView {
    pub did: String,
    pub handle: String,
    pub displayName: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub indexedAt: Option<String>,
    pub viewer: Option<ViewerState>,
    pub labels: Option<Vec<Label>>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct ViewerState {
    pub muted: Option<bool>,
    pub mutedByList: Option<Value>, // TODO
    pub blockedBy: Option<bool>,
    pub blocking: Option<String>,
    pub following: Option<String>,
    pub followedBy: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct ProfileViewBasic {
    pub did: String,
    pub handle: String,
    pub displayName: Option<String>,
    pub avatar: Option<String>,
    pub viewer: Option<Value>,
    pub labels: Option<Vec<Label>>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct ProfileViewDetailed {
    pub did: String,
    pub handle: String,
    pub displayName: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub followersCount: u64,
    pub followsCount: u64,
    pub postsCount: u64,
    pub indexedAt: Option<String>,
    pub viewer: Option<ViewerState>,
    pub labels: Option<Vec<Label>>,
}

/// for Timeline or AuthorFeed
#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct GenericFeed {
    pub cursor: Option<String>,
    pub feed: Vec<FeedViewPost>,
}
/* XXX:
#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct User {
    pub did: String,
    pub handle: String,
    pub displayName: Option<String>,
}
*/

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct PostView {
    pub uri: String,
    pub cid: String,
    pub author: ProfileViewBasic,
    pub record: PostRecord,
    pub embed: Option<PostEmbedView>,
    pub replyCount: u64,
    pub repostCount: u64,
    pub likeCount: u64,
    pub indexedAt: String,
    pub viewer: Option<Value>,
    pub labels: Option<Vec<Label>>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct ThreadViewPost {
    // TODO: doing this as the intersetion of #threadViewPost and #notFoundPost. actually it is
    // supposed to be a union type
    // #notFoundPost fields (uri and notFound actually required)
    pub uri: Option<String>,
    pub notFound: Option<bool>,
    // #blockedPost fields (uri and blocked actually required)
    pub blocked: Option<bool>,
    pub author: Option<Value>,
    // #threadViewPost fields (post actually required)
    pub post: Option<PostView>,
    pub parent: Option<Box<ThreadViewPost>>,
    pub replies: Option<Vec<ThreadViewPost>>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct FeedViewPost {
    pub post: PostView,
    pub reply: Option<ReplyRef>,
    // TODO: this could extend to other "reasons" in the future
    pub reason: Option<ReasonRepost>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct ReasonRepost {
    pub by: ProfileViewBasic,
    pub indexedAt: String,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct PostRecord {
    pub text: String,
    pub entities: Option<Vec<PostEntity>>,
    pub facets: Option<Vec<RichtextFacet>>,
    pub reply: Option<ReplyRef>,
    pub embed: Option<PostEmbed>,
    pub langs: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub createdAt: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct ReplyRef {
    // TODO: these should be StrongRef
    pub parent: Subject,
    pub root: Subject,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct RichtextFacet {
    pub index: ByteSlice,
    pub features: Vec<FacetFeature>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct FacetFeature {
    // TODO: this is a hack; actually separate mention and link types
    pub did: Option<String>,
    pub uri: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct ByteSlice {
    pub byteStart: u64,
    pub byteEnd: u64,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct PostEntity {
    pub index: TextSlice,
    pub r#type: String,
    pub value: String,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct TextSlice {
    pub start: u64,
    pub end: u64,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct PostEmbed {
    pub external: Option<EmbedExternal>,
    pub images: Option<Vec<EmbedImage>>,
    pub record: Option<StrongRef>,
    pub recordWithMedia: Option<Value>, // TODO
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct PostEmbedView {
    pub images: Option<Vec<EmbedImageView>>,
    pub external: Option<EmbedExternalView>,
    pub record: Option<Value>,          // TODO
    pub recordWithMedia: Option<Value>, // TODO
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct EmbedExternal {
    pub uri: String,
    pub title: String,
    pub description: String,
    pub thumb: Option<Blob>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct EmbedExternalView {
    pub uri: String,
    pub title: String,
    pub description: String,
    pub thumb: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct EmbedImage {
    pub image: Blob,
    pub alt: String,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct Blob {
    #[serde(rename = "$type")]
    pub blob_type: Option<String>,
    #[serde(rename = "ref")]
    pub link: Option<CidLink>,
    pub cid: Option<String>, // deprecated
    pub mimeType: String,
    pub size: Option<u64>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct CidLink {
    #[serde(rename = "$link")]
    pub cid: String,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct EmbedImageView {
    pub thumb: String,
    pub fullsize: String,
    pub alt: String,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct PostThread {
    pub thread: ThreadViewPost,
}

/* XXX
#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct FollowTarget {
    // TODO: nested follow list?
    pub subject: Subject,
    pub did: String,
    pub handle: String,
    pub displayName: Option<String>,
    pub createdAt: Option<String>,
    pub indexedAt: String,
}
*/

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct FollowList {
    pub subject: Subject,
    pub cursor: Option<String>,
    pub follows: Vec<ProfileView>,
}
