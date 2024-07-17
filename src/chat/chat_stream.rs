use crate::adapter::inter_stream::{InterStreamEnd, InterStreamEvent};
use crate::chat::MetaUsage;
use crate::Result;
use derive_more::From;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

type InterStreamType = Pin<Box<dyn Stream<Item = Result<InterStreamEvent>>>>;

pub struct ChatStream {
	inter_stream: InterStreamType,
}

impl ChatStream {
	pub fn new(inter_stream: InterStreamType) -> Self {
		ChatStream { inter_stream }
	}

	pub fn from_inter_stream<T>(inter_stream: T) -> Self
	where
		T: Stream<Item = Result<InterStreamEvent>> + Unpin + 'static,
	{
		let boxed_stream: InterStreamType = Box::pin(inter_stream);
		ChatStream::new(boxed_stream)
	}
}

// region:    --- Stream Impl

impl Stream for ChatStream {
	type Item = Result<ChatStreamEvent>;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let this = self.get_mut();

		match Pin::new(&mut this.inter_stream).poll_next(cx) {
			Poll::Ready(Some(Ok(event))) => {
				let chat_event = match event {
					InterStreamEvent::Start => ChatStreamEvent::Start,
					InterStreamEvent::Chunk(content) => ChatStreamEvent::Chunk(StreamChunk { content }),
					InterStreamEvent::End(inter_end) => ChatStreamEvent::End(inter_end.into()),
				};
				Poll::Ready(Some(Ok(chat_event)))
			}
			Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
			Poll::Ready(None) => Poll::Ready(None),
			Poll::Pending => Poll::Pending,
		}
	}
}

// endregion: --- Stream Impl

// region:    --- ChatStreamEvent

#[derive(Debug, From)]
pub enum ChatStreamEvent {
	Start,
	Chunk(StreamChunk),
	End(StreamEnd),
}

#[derive(Debug)]
pub struct StreamChunk {
	pub content: String,
}

#[derive(Debug, Default)]
pub struct StreamEnd {
	/// The eventual capture UsageMeta
	pub captured_usage: Option<MetaUsage>,

	/// The optional captured full content
	pub captured_content: Option<String>,
}

impl From<InterStreamEnd> for StreamEnd {
	fn from(inter_end: InterStreamEnd) -> Self {
		StreamEnd {
			captured_usage: inter_end.captured_usage,
			captured_content: inter_end.captured_content,
		}
	}
}

// endregion: --- ChatStreamEvent
