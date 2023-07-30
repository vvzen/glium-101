use glium::{glutin, implement_vertex, Surface};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

const VERTEX_SHADER_SRC: &str = r#"
#version 140

in vec2 position;

void main() {
    vec2 pos = position;
    gl_Position = vec4(pos, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER_SRC: &str = r#"
#version 140

out vec4 color;
uniform vec4 requested_rgba_color;

void main() {
    color = requested_rgba_color;
}
"#;

// Custom structs required to provide a more friendly
// abstraction on top of the inner working of OpenGL
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    fn as_tuple(self) -> (f32, f32, f32, f32) {
        return (self.r, self.g, self.b, self.a);
    }
}

#[non_exhaustive]
enum ShapePrimitive {
    Circle,
    Triangle,
}

struct SketchDrawCommand<'a> {
    vertex_buffer: glium::VertexBuffer<Vertex>,
    indices: glium::index::NoIndices,
    uniforms:
        glium::uniforms::UniformsStorage<'a, (f32, f32, f32, f32), glium::uniforms::EmptyUniforms>,
    draw_parameters: glium::draw_parameters::DrawParameters<'a>,
}

/// ``color`` will be the fill color of our shape
/// ``vertices`` should contain the exact number of vertices
/// that will be composing our shape
fn generate_draw_command(
    display: &glium::Display,
    vertices: Vec<Vertex>,
    primitive: ShapePrimitive,
    color: Color,
    add_fill: bool,
    stroke_width: Option<f32>,
) -> SketchDrawCommand {
    // FIXME: return the commands needed for both the fill and the stroke

    let rgba_color = color.as_tuple();

    // Vertex buffers are the basic ingredients that will be uploaded to the GPU
    let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();

    // Tell OpenGL how to link together the vertices that we will pass
    let primitive_type = match primitive {
        ShapePrimitive::Circle => glium::index::PrimitiveType::LineLoop,
        ShapePrimitive::Triangle => glium::index::PrimitiveType::TrianglesList,
    };

    let indices = glium::index::NoIndices(primitive_type);

    // A uniform that will be passed to our shader
    let uniforms = glium::uniform! {
        requested_rgba_color: rgba_color,
    };

    let draw_parameters = glium::draw_parameters::DrawParameters {
        multisampling: true,
        polygon_mode: match add_fill {
            true => glium::PolygonMode::Fill,
            false => glium::PolygonMode::Line,
        },
        line_width: stroke_width,
        ..Default::default()
    };

    SketchDrawCommand {
        vertex_buffer,
        indices,
        uniforms,
        draw_parameters,
    }
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new().with_title("glium 101");

    let context_builder = glutin::ContextBuilder::new().with_multisampling(16);
    let display = glium::Display::new(window_builder, context_builder, &event_loop).unwrap();

    implement_vertex!(Vertex, position);

    // Event Loop for the Window
    event_loop.run(move |event, _, control_flow| {
        /*
        When we defined the Vertex struct in our shape, we created a field named 'position'
        which contains the position of our vertex. But contrary to what I let you think,
        this struct doesn't contain the actual position of the vertex but only an attribute
        whose value is passed to the vertex shader. OpenGL doesn't care about the name of
        the attribute, all it does is passing its value to the vertex shader.
        The 'in vec2 position;' line of our shader is here to declare that we are expected
        to be passed an attribute named position whose type is vec2
        (which corresponds to [f32; 2] in Rust).
        The main function of our shader is called once per vertex, which means three times
        for our triangle. The first time, the value of position will be [-0.5, -0.5], the
        second time it will be [0, 0.5], and the third time [0.5, -0.25]. It is in this
        function that we actually tell OpenGL what the position of our vertex is, thanks
        to the gl_Position = vec4(position, 0.0, 1.0); line.
        We need to do a small conversion because OpenGL doesn't expect two-dimensional
        coordinates, but four-dimensional coordinates (the reason for this will be covered in
        a later tutorial).
        */

        let geometry_shader = None;

        let program = glium::Program::from_source(
            &display,
            VERTEX_SHADER_SRC,
            FRAGMENT_SHADER_SRC,
            geometry_shader,
        )
        .unwrap();

        // Here we do the actual drawing into the frame
        let mut frame = display.draw();

        // Clear the background
        frame.clear_color(1.0, 1.0, 1.0, 1.0);

        // Here we draw our custom shape by sending the vertices and the shaders
        // The 'draw command' (which contains all of the instructions for drawing)
        // is generated programmatically based on the primitive that we need to render
        let vertices = vec![
            Vertex {
                position: [-0.5, -0.5],
            },
            Vertex {
                position: [0.0, 0.5],
            },
            Vertex {
                position: [0.5, -0.25],
            },
        ];

        // Fill of the triangle
        let add_fill = true;
        let stroke_width = None;
        let triangle_color = Color::new(1.0, 0.0, 0.0, 0.0);
        let triangle_draw_command = generate_draw_command(
            &display,
            vertices.clone(),
            ShapePrimitive::Triangle,
            triangle_color,
            add_fill,
            stroke_width,
        );

        frame
            .draw(
                &triangle_draw_command.vertex_buffer,
                &triangle_draw_command.indices,
                &program,
                &triangle_draw_command.uniforms,
                &triangle_draw_command.draw_parameters,
            )
            .unwrap();

        // Stroke of the triangle
        let add_fill = false;
        let stroke_width = Some(4.0 as f32);
        let triangle_color = Color::new(1.0, 1.0, 0.0, 0.0);
        let triangle_draw_command = generate_draw_command(
            &display,
            vertices.clone(),
            ShapePrimitive::Triangle,
            triangle_color,
            add_fill,
            stroke_width,
        );

        frame
            .draw(
                &triangle_draw_command.vertex_buffer,
                &triangle_draw_command.indices,
                &program,
                &triangle_draw_command.uniforms,
                &triangle_draw_command.draw_parameters,
            )
            .unwrap();

        frame.finish().unwrap();

        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);

        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match event {
            glutin::event::Event::WindowEvent { window_id, event } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => (),
        }
    });
}
