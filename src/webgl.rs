use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

pub struct CanvasProperties {
  pub height: f32,
  pub width: f32,
}

pub fn get_context_by_id(id: &str) -> Result<(WebGl2RenderingContext, CanvasProperties), String> {
  let document = web_sys::window().unwrap().document().unwrap();

  let canvas: web_sys::HtmlCanvasElement = document
    .get_element_by_id(id)
    .unwrap()
    .dyn_into::<web_sys::HtmlCanvasElement>()
    .unwrap();

  let context = canvas
    .get_context("webgl2")
    .unwrap()
    .unwrap()
    .dyn_into::<WebGl2RenderingContext>()
    .unwrap();

  let properties = CanvasProperties {
    height: canvas.client_height() as f32,
    width: canvas.client_width() as f32,
  };

  Ok((context, properties))
}

pub fn compile_shader(
  context: &WebGl2RenderingContext,
  shader_type: u32,
  source: &str,
) -> Result<WebGlShader, String> {
  let shader = context
    .create_shader(shader_type)
    .ok_or_else(|| String::from("Unable to create shader object"))?;

  context.shader_source(&shader, source);
  context.compile_shader(&shader);

  let compile_success = context
    .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
    .as_bool()
    .unwrap_or(false);

  if compile_success {
    Ok(shader)
  } else {
    let msg = context
      .get_shader_info_log(&shader)
      .unwrap_or_else(|| String::from("Unknown error creating shader"));
    Err(msg)
  }
}

pub fn link_shader_program(
  context: &WebGl2RenderingContext,
  vertex_shader: &WebGlShader,
  fragment_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
  let program = context
    .create_program()
    .ok_or_else(|| String::from("Unable to create shader object"))?;

  context.attach_shader(&program, vertex_shader);
  context.attach_shader(&program, fragment_shader);
  context.link_program(&program);

  let link_success = context
    .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
    .as_bool()
    .unwrap_or(false);

  if link_success {
    Ok(program)
  } else {
    let msg = context
      .get_program_info_log(&program)
      .unwrap_or_else(|| String::from("Unknown error creating program object"));
    Err(msg)
  }
}
