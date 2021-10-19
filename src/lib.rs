use wasm_bindgen::prelude::*;
// use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation};
extern crate nalgebra_glm as glm;

mod webgl;
use webgl::{compile_shader, get_context_by_id, link_shader_program};

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
  // let document = web_sys::window().unwrap().document().unwrap();
  // let canvas = document.get_element_by_id("canvas").unwrap();
  // let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
  let (context, canvas) = get_context_by_id("canvas").unwrap();

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

  let info = ShaderInfo {
    program: &program,
    canvas_height: canvas.height,
    canvas_width: canvas.width,
    vertex_position,
    program_projection_matrix: &projection_matrix,
    program_model_view_matrix: &model_view_matrix,
  };

  draw(&context, &info, &buffers);

  Ok(())
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

struct ShaderInfo<'a> {
  program: &'a WebGlProgram,
  canvas_width: f32,
  canvas_height: f32,
  vertex_position: u32,
  program_projection_matrix: &'a WebGlUniformLocation,
  program_model_view_matrix: &'a WebGlUniformLocation,
}

fn draw(context: &WebGl2RenderingContext, info: &ShaderInfo, position_buffer: &WebGlBuffer) {
  context.clear_color(0.0, 0.0, 0.0, 1.0);
  context.clear_depth(1.0);
  context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
  context.enable(WebGl2RenderingContext::DEPTH_TEST);
  context.depth_func(WebGl2RenderingContext::LEQUAL);

  context
    .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

  let field_of_view = 45.0 * std::f32::consts::PI / 180.0;
  let aspect = info.canvas_width / info.canvas_height;
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
      info.vertex_position,
      num_components,
      data_type,
      normalize,
      stride,
      offset,
    );

    context.enable_vertex_attrib_array(info.vertex_position);
  }

  context.use_program(Some(info.program));

  context.uniform_matrix4fv_with_f32_array(
    Some(info.program_projection_matrix),
    false,
    &vec_projection_matrix,
  );

  context.uniform_matrix4fv_with_f32_array(
    Some(info.program_model_view_matrix),
    false,
    &vec_model_view_matrix,
  );

  let offset = 0;
  let vertex_count = 4;
  // let data_type = WebGl2RenderingContext::UNSIGNED_SHORT;
  context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, offset, vertex_count);
}
