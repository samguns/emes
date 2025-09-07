use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{CallToolResult, Content};
#[cfg(target_os = "linux")]
{
    #[link(name = "qwen2_vl_cpp")]
    unsafe extern "C" {
        unsafe fn greet();
    }

    #[derive(Clone)]
    pub struct Qwen2VLService {
        tool_router: ToolRouter<Self>,
    }

    #[rmcp::tool_router]
    impl Qwen2VLService {
        pub fn new() -> Self {
            Self {
                tool_router: Self::tool_router(),
            }
        }

        #[rmcp::tool(description = "Greet the user")]
        async fn greet(&self) -> Result<CallToolResult, rmcp::ErrorData> {
            unsafe { greet() };

            Ok(CallToolResult::success(vec![Content::text(
                "Hello, world!".to_string(),
            )]))
        }
    }

    #[rmcp::tool_handler]
    impl rmcp::ServerHandler for Qwen2VLService {
        fn get_info(&self) -> rmcp::model::ServerInfo {
            rmcp::model::ServerInfo {
                capabilities: rmcp::model::ServerCapabilities::builder()
                    .enable_tools()
                    .build(),
                instructions: Some(
                    "Qwen2VLService is a service that provides a tool for Qwen2VL.".to_string(),
                ),
                ..Default::default()
            }
        }
    }
}
