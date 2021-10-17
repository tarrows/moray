use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
  WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlUniformLocation,
};
extern crate nalgebra_glm as glm;

static VERTEX_SHADER: &'static str = r#"
  attribute vec4 aVertexPosition;

  uniform mat4 uModelViewMatrix;
  uniform mat4 uProjectionMatrix;

  void main() {
    gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
  }
"#;

static FRAGMENT_SHADER: &'static str = r#"
  void main() {
    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
  }
"#;

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
  let document = web_sys::window().unwrap().document().unwrap();
  let canvas = document.get_element_by_id("canvas").unwrap();
  let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
  let context = canvas
    .get_context("webgl2")?
    .unwrap()
    .dyn_into::<WebGl2RenderingContext>()?;

  let vertex_shader = compile_shader(
    &context,
    WebGl2RenderingContext::VERTEX_SHADER,
    VERTEX_SHADER,
  )?;
  let fragment_shader = compile_shader(
    &context,
    WebGl2RenderingContext::FRAGMENT_SHADER,
    FRAGMENT_SHADER,
  )?;

  let program = link_shader_program(&context, &vertex_shader, &fragment_shader)?;
  context.use_program(Some(&program));

  let vertex_position = context.get_attrib_location(&program, "aVertexPosition") as u32;
  let model_view_matrix = context
    .get_uniform_location(&program, "uModelViewMatrix")
    .unwrap();
  let projection_matrix = context
    .get_uniform_location(&program, "uProjectionMatrix")
    .unwrap();

  let buffers = init_buffers(&context);

  let vao = context
    .create_vertex_array()
    .ok_or("Could not create vertex array object")?;
  context.bind_vertex_array(Some(&vao));
  context.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
  context.enable_vertex_attrib_array(vertex_position);
  context.bind_vertex_array(Some(&vao));

  draw(
    &context,
    &program,
    canvas.client_width() as f32,
    canvas.client_height() as f32,
    vertex_position,
    &projection_matrix,
    &model_view_matrix,
    &buffers,
  );

  Ok(())
}

fn compile_shader(
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

fn link_shader_program(
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

fn init_buffers(context: &WebGl2RenderingContext) -> WebGlBuffer {
  let positions = [-1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0];

  let position_buffer = context
    .create_buffer()
    .ok_or("Failed to create buffer")
    .unwrap();
  context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));

  unsafe {
    let positions_array_buffer_view = js_sys::Float32Array::view(&positions);

    context.buffer_data_with_array_buffer_view(
      WebGl2RenderingContext::ARRAY_BUFFER,
      &positions_array_buffer_view,
      WebGl2RenderingContext::STATIC_DRAW,
    );
  }

  position_buffer
}

fn draw(
  context: &WebGl2RenderingContext,
  shader_program: &WebGlProgram,
  canvas_width: f32,
  canvas_height: f32,
  vertex_position: u32,
  program_projection_matrix: &WebGlUniformLocation,
  program_model_view_matrix: &WebGlUniformLocation,
  position_buffer: &WebGlBuffer,
) {
  context.clear_color(0.0, 0.0, 0.0, 1.0);
  context.clear_depth(1.0);
  context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
  context.enable(WebGl2RenderingContext::DEPTH_TEST);
  context.depth_func(WebGl2RenderingContext::LEQUAL);

  context
    .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

  let field_of_view = 45.0 * std::f32::consts::PI / 180.0;
  let aspect = canvas_width / canvas_height;
  let z_near = 0.1;
  let z_far = 100.0;

  let projection_matrix = glm::perspective(aspect, field_of_view, z_near, z_far);
  let vec_projection_matrix = projection_matrix.iter().map(|v| *v).collect::<Vec<_>>();

  let model_view_matrix = glm::translate(&glm::Mat4::identity(), &glm::TVec3::new(-0.0, 0.0, -6.0));
  let vec_model_view_matrix = model_view_matrix.iter().map(|v| *v).collect::<Vec<_>>();

  {
    let num_components = 2;
    let data_type: u32 = WebGl2RenderingContext::FLOAT;
    let normalize = false;
    let stride = 0;
    let offset = 0;

    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));
    context.vertex_attrib_pointer_with_i32(
      vertex_position,
      num_components,
      data_type,
      normalize,
      stride,
      offset,
    );

    context.enable_vertex_attrib_array(vertex_position);
  }

  context.use_program(Some(&shader_program));

  context.uniform_matrix4fv_with_f32_array(
    Some(program_projection_matrix),
    false,
    &vec_projection_matrix,
  );

  context.uniform_matrix4fv_with_f32_array(
    Some(program_model_view_matrix),
    false,
    &vec_model_view_matrix,
  );

  let offset = 0;
  let vertex_count = 4;
  // let data_type = WebGl2RenderingContext::UNSIGNED_SHORT;
  context.draw_arrays(WebGl2RenderingContext::TRIANGLES, offset, vertex_count);
}
