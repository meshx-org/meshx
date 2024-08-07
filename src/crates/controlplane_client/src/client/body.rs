use std::{
    error::Error as StdError,
    fmt,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use futures::{Stream, TryStreamExt};
use bytes::Bytes;
use http_body::{Body as HttpBody, Frame, SizeHint};
use http_body_util::{combinators::UnsyncBoxBody, BodyExt, BodyStream};

/// A request body.
pub struct Body {
    kind: Kind,
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Body");
        match self.kind {
            Kind::Once(_) => builder.field("kind", &"Once"),
            Kind::Wrap(_) => builder.field("kind", &"Wrap"),
        };
        builder.finish()
    }
}

enum Kind {
    Once(Option<Bytes>),
    Wrap(UnsyncBoxBody<Bytes, Box<dyn StdError + Send + Sync>>),
}

impl Body {
    fn new(kind: Kind) -> Self {
        Body { kind }
    }

    /// Create an empty body
    pub fn empty() -> Self {
        Self::new(Kind::Once(None))
    }

    // Create a body from an existing body
    pub(crate) fn wrap_body<B>(body: B) -> Self
    where
        B: HttpBody<Data = Bytes> + Send + 'static,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Body::new(Kind::Wrap(body.map_err(Into::into).boxed_unsync()))
    }

    // Convert this body into `Stream` which iterates only data frame skipping the other kind of frame
    pub(crate) fn into_data_stream(self) -> impl Stream<Item = Result<Bytes, crate::Error>> {
        Box::pin(BodyStream::new(self).try_filter_map(|frame| async { Ok(frame.into_data().ok()) }))
    }
}

impl From<Bytes> for Body {
    fn from(bytes: Bytes) -> Self {
        if bytes.is_empty() {
            Self::empty()
        } else {
            Self::new(Kind::Once(Some(bytes)))
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(vec: Vec<u8>) -> Self {
        Self::from(Bytes::from(vec))
    }
}

impl HttpBody for Body {
    type Data = Bytes;
    type Error = crate::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match &mut self.kind {
            Kind::Once(val) => Poll::Ready(val.take().map(|bytes| Ok(Frame::data(bytes)))),
            Kind::Wrap(body) => pin!(body).poll_frame(cx).map_err(crate::error::Error::Service),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match &self.kind {
            Kind::Once(Some(bytes)) => SizeHint::with_exact(bytes.len() as u64),
            Kind::Once(None) => SizeHint::with_exact(0),
            Kind::Wrap(body) => body.size_hint(),
        }
    }

    fn is_end_stream(&self) -> bool {
        match &self.kind {
            Kind::Once(Some(bytes)) => bytes.is_empty(),
            Kind::Once(None) => true,
            Kind::Wrap(body) => body.is_end_stream(),
        }
    }
}