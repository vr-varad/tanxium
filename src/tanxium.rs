use std::rc::Rc;

use boa_engine::builtins::promise::PromiseState;
use boa_engine::context::ContextBuilder;
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{js_str, js_string, Context, JsResult, JsValue, Source};

use crate::builtins;
use crate::runtime;
use crate::typescript;

use typescript::{transpile_typescript, YASUMU_WORKSPACE_SCRIPT_NAME, YASUMU_WORKSPACE_SCRIPT_URL};

pub struct ScriptExtension {
    path: String,
    transpile: bool,
}

pub struct Tanxium {
    pub context: Context,
}

impl Tanxium {
    pub fn new() -> Result<Self, std::io::Error> {
        let job_queue = Rc::new(runtime::event_loop::EventLoop::new());
        let module_loader = Rc::new(runtime::module_loader::YasumuModuleLoader {});
        let context = ContextBuilder::new()
            .job_queue(job_queue)
            .module_loader(module_loader)
            .build()
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create context: {}", e),
                )
            })?;

        Ok(Tanxium { context })
    }

    pub fn load_extensions(&mut self, ext: Vec<ScriptExtension>) -> Result<(), std::io::Error> {
        for e in ext {
            let content = std::fs::read_to_string(e.path.as_str())?;

            let js_src = if e.transpile {
                self.transpile(content.as_str())?
            } else {
                content
            };

            let src = Source::from_bytes(js_src.as_bytes());
            self.context
                .eval(src)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        }

        Ok(())
    }

    pub fn eval(&mut self, code: &str) -> JsResult<JsValue> {
        let src = Source::from_bytes(code.as_bytes());
        self.context.eval(src)
    }

    pub fn transpile(&mut self, code: &str) -> Result<String, std::io::Error> {
        let transpiled = transpile_typescript(code);

        match transpiled {
            Ok(js) => Ok(js),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        }
    }

    pub fn run_event_loop(&mut self) {
        self.context.run_jobs();
    }
}

// pub async fn evaluate_javascript(
//     app: tauri::AppHandle,
//     code: &str,
//     prepare: &str,
//     id: &str,
//     typescript: Option<bool>,
//     test: Option<bool>,
//     workspace_state: tauri::State<'_, WorkspaceState>,
// ) -> Result<String, String> {
//     let code = code.to_string();
//     let prepare = prepare.to_string();
//     let id = id.to_string();
//     let current_workspace = workspace_state.get_current_workspace();

//     let handle = smol::spawn(async move {
//         let ts_supported = typescript.is_some() && typescript.unwrap().eq(&true);
//         let prepare_script = if ts_supported {
//             let res = transpile_typescript(&prepare);

//             res.unwrap()
//         } else {
//             prepare
//         };

//         let final_code = if ts_supported {
//             let res = transpile_typescript(&code);

//             res.unwrap()
//         } else {
//             code
//         };
//         let src = Source::from_bytes(final_code.as_bytes());
//         let cwd = match current_workspace.clone() {
//             Some(ws) => ws,
//             None => std::env::current_dir()
//                 .unwrap()
//                 .as_mut_os_string()
//                 .clone()
//                 .into_string()
//                 .unwrap(),
//         };

//         let (mut ctx, module) =
//             runtime::init_runtime_and_event_loop(cwd.clone(), src, ts_supported.clone());

//         builtins::performance::performance_init(&mut ctx);
//         builtins::crypto::crypto_init(&mut ctx);
//         builtins::yasumu::runtime_init(
//             &mut ctx,
//             current_workspace.clone(),
//             app.clone(),
//             id.clone(),
//             ts_supported,
//             test.unwrap_or(false),
//         );

//         // init runtime apis
//         setup_runtime(&mut ctx, app);

//         ctx.eval(Source::from_bytes(prepare_script.as_bytes()))
//             .unwrap();

//         let import_meta_env = ObjectInitializer::new(&mut ctx)
//             .property(js_str!("WORKSPACE_ID"), js_string!(id), Attribute::all())
//             .property(
//                 js_str!("TEST"),
//                 JsValue::Boolean(test.unwrap_or(false)),
//                 Attribute::all(),
//             )
//             .build();

//         let import_meta = ObjectInitializer::new(&mut ctx)
//             .property(
//                 js_str!("url"),
//                 js_string!(YASUMU_WORKSPACE_SCRIPT_URL),
//                 Attribute::all(),
//             )
//             .property(
//                 js_str!("filename"),
//                 js_string!(YASUMU_WORKSPACE_SCRIPT_NAME),
//                 Attribute::all(),
//             )
//             .property(js_str!("dirname"), js_string!(cwd), Attribute::all())
//             .property(js_str!("env"), import_meta_env, Attribute::all())
//             .build();

//         ctx.module_loader()
//             .init_import_meta(&import_meta, &module, &mut ctx);

//         let promise = module.load_link_evaluate(&mut ctx);

//         ctx.run_jobs();

//         let output = match promise.state() {
//             PromiseState::Pending => Err("Module failed to execute".to_string()),
//             PromiseState::Fulfilled(_) => {
//                 let global_obj = ctx.global_object();
//                 let yasumu = global_obj.get(js_string!("Yasumu"), &mut ctx).unwrap();
//                 let yasumu_obj = yasumu
//                     .as_object()
//                     .ok_or("Failed to convert Yasumu to object")?;
//                 let context = yasumu_obj.get(js_string!("context"), &mut ctx).unwrap();
//                 let context_obj = context
//                     .as_object()
//                     .ok_or("Failed to convert context to object")?;
//                 let meta = context_obj.get(js_string!("__meta"), &mut ctx).unwrap();
//                 let meta_obj = meta.to_json(&mut ctx).unwrap();
//                 Ok(format!("{}", meta_obj.to_string()))
//             }
//             PromiseState::Rejected(err) => Err(format!("{}", err.display())),
//         };

//         println!("Output: {:?}", output);

//         output
//     });

//     match handle.await {
//         Ok(res) => Ok(res),
//         Err(e) => Err(format!("FatalError: {}", e)),
//     }
// }
