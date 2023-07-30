use glium::{glutin, implement_vertex, Surface};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new().with_title("glium 101");

    let context_builder = glutin::ContextBuilder::new().with_multisampling(16);
    let display = glium::Display::new(window_builder, context_builder, &event_loop).unwrap();

    implement_vertex!(Vertex, position);

    // A uniform that will be passed to our shader
    // The vertices that will be composing our shape
    let shape = vec![
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

    // Event Loop for the Window
    event_loop.run(move |event, _, control_flow| {
        // Vertex buffers are the basic ingredients that will be uploaded to the GPU
        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

        // Tell OpenGL how to link together the vertices that we will pass
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

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

        let vertex_shader_src = r#"
        #version 140

        in vec2 position;

        void main() {
            vec2 pos = position;
            gl_Position = vec4(pos, 0.0, 1.0);
        }
        "#;

        let fragment_shader_src = r#"
        #version 140

        out vec4 color;
        uniform vec4 triangle_rgba;

        void main() {
            //color = vec4(1.0, 0.0, 0.0, 1.0);
            color = triangle_rgba;
        }
        "#;

        let geometry_shader = None;

        let program = glium::Program::from_source(
            &display,
            vertex_shader_src,
            fragment_shader_src,
            geometry_shader,
        )
        .unwrap();

        let triangle_rgba = (1.0f32, 1.0f32, 0.0f32, 0.0f32);

        let uniforms = glium::uniform! {
            triangle_rgba: triangle_rgba,
        };

        // Here we do the actual drawing into the frame
        let mut frame = display.draw();

        // Clear the background
        frame.clear_color(0.0, 0.0, 1.0, 1.0);

        // Draw our custom shape by sending the vertices and the shaders
        // TODO: DrawParameters
        // https://docs.rs/glium/latest/glium/draw_parameters/index.html
        let draw_parameters = glium::draw_parameters::DrawParameters {
            multisampling: true,
            ..Default::default()
        };
        frame
            .draw(
                &vertex_buffer,
                &indices,
                &program,
                &uniforms,
                &draw_parameters,
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
