use crate::adapter::gemini::GeminiStream;
use crate::adapter::support::get_api_key_resolver;
use crate::adapter::{Adapter, AdapterConfig, AdapterKind, ServiceType, WebRequestData};
use crate::chat::{ChatRequest, ChatRequestOptions, ChatResponse, ChatRole, ChatStream, ChatStreamResponse};
use crate::utils::x_value::XValue;
use crate::webc::{WebResponse, WebStream};
use crate::{ConfigSet, Error, Result};
use reqwest::RequestBuilder;
use serde_json::{json, Value};
use std::sync::OnceLock;

pub struct GeminiAdapter;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/";

// curl \
//   -H 'Content-Type: application/json' \
//   -d '{"contents":[{"parts":[{"text":"Explain how AI works"}]}]}' \
//   -X POST 'https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash-latest:generateContent?key=YOUR_API_KEY'

impl Adapter for GeminiAdapter {
	fn default_adapter_config(_kind: AdapterKind) -> &'static AdapterConfig {
		static INSTANCE: OnceLock<AdapterConfig> = OnceLock::new();
		INSTANCE.get_or_init(|| AdapterConfig::default().with_auth_env_name("GEMINI_API_KEY"))
	}

	fn get_service_url(_kind: AdapterKind, service_type: ServiceType) -> String {
		match service_type {
			ServiceType::Chat | ServiceType::ChatStream => BASE_URL.to_string(),
		}
	}

	fn to_web_request_data(
		kind: AdapterKind,
		config_set: &ConfigSet<'_>,
		service_type: ServiceType,
		model: &str,
		chat_req: ChatRequest,
		_chat_req_options: Option<&ChatRequestOptions>,
	) -> Result<WebRequestData> {
		let api_key = get_api_key_resolver(kind, config_set)?;

		// For gemini, the service url returned is just the base url
		// since model and API key is part of the url (see below)
		let url = Self::get_service_url(kind, service_type);

		// e.g., '...models/gemini-1.5-flash-latest:generateContent?key=YOUR_API_KEY'
		let url = match service_type {
			ServiceType::Chat => format!("{url}models/{model}:generateContent?key={api_key}"),
			ServiceType::ChatStream => format!("{url}models/{model}:streamGenerateContent?key={api_key}"),
		};

		let headers = vec![];

		let GeminiChatRequestParts { contents } = into_gemini_request_parts(kind, chat_req)?;

		let payload = json!({
			"contents": contents,
		});

		Ok(WebRequestData { url, headers, payload })
	}

	fn to_chat_response(_kind: AdapterKind, web_response: WebResponse) -> Result<ChatResponse> {
		let WebResponse { body, .. } = web_response;

		let gemini_response = body_to_gemini_chat_response(body)?;

		Ok(ChatResponse {
			content: gemini_response.content,
			..Default::default()
		})
	}

	fn to_chat_stream(_kind: AdapterKind, reqwest_builder: RequestBuilder) -> Result<ChatStreamResponse> {
		let web_stream = WebStream::new_with_pretty_json_array(reqwest_builder);

		let gemini_stream = GeminiStream::new(web_stream);
		let chat_stream = ChatStream::from_inter_stream(gemini_stream);

		Ok(ChatStreamResponse { stream: chat_stream })
	}
}

// region:    --- Support

// stuct Gemini

pub struct GeminiChatResponse {
	pub content: Option<String>,
}

pub fn body_to_gemini_chat_response(mut body: Value) -> Result<GeminiChatResponse> {
	let content = body.x_take::<Value>("/candidates/0/content/parts/0/text")?;

	Ok(GeminiChatResponse {
		content: content.as_str().map(String::from),
	})
}

struct GeminiChatRequestParts {
	/// The chat history (user and assistant, except last user message which is message)
	contents: Vec<Value>,
}

/// Takes the genai ChatMessages and build the System string and json Messages for gemini.
/// See: https://ai.google.dev/api/rest/v1/models/generateContent
fn into_gemini_request_parts(adapter_kind: AdapterKind, chat_req: ChatRequest) -> Result<GeminiChatRequestParts> {
	let mut contents: Vec<Value> = Vec::new();

	// -- Build
	for msg in chat_req.messages {
		let content = msg.content;
		match msg.role {
			// for now, system go as "user" (later, we might have adapter_config.system_to_user_tmpl)
			ChatRole::System => contents.push(json! ({ "role": "user", "parts": [{"text": content}]})),
			ChatRole::User => contents.push(json! ({"role": "user", "parts": [{"text": content}]})),
			ChatRole::Assistant => contents.push(json! ({"role": "model", "parts": [{"text": content}]})),
			ChatRole::Tool => {
				return Err(Error::AdapterMessageRoleNotSupport {
					adapter_kind,
					role: ChatRole::Tool,
				})
			}
		}
	}

	Ok(GeminiChatRequestParts { contents })
}

// endregion: --- Support
