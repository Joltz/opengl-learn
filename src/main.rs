use gl33::global_loader::*;
use gl33::global_loader::{glClear, load_global_gl};
use gl33::*;
use glutin::dpi::{LogicalPosition, Position};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use glutin::Api;
use std::mem::size_of;
use std::mem::size_of_val;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("Lyra-OpenGL3.3")
        .with_inner_size(glutin::dpi::LogicalSize::new(512.0, 512.0))
        .with_position(Position::Logical(LogicalPosition::new(1600.0, 200.0)));
    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_vsync(true)
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    let context = unsafe { context.make_current().unwrap() };
    println!(
        "Pixel fomrat of the window's GL context: {:?}",
        context.get_pixel_format()
    );

    unsafe {
        // Load in all our OpenGL Functions
        load_global_gl(&|ptr| {
            let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
            let r_str = c_str.to_str().unwrap();
            context.get_proc_address(r_str) as _
        });

        // Vertex shader
        const VERT_SHADER_SRC: &str =
            "#version 330 core\nlayout (location = 0) in vec3 aPos;\n\nvoid main()\n{\n    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);\n}";
        let vertex_shader = glCreateShader(GL_VERTEX_SHADER);
        glShaderSource(
            vertex_shader,
            1,
            &(VERT_SHADER_SRC.as_bytes().as_ptr().cast()),
            &(VERT_SHADER_SRC.len().try_into()?),
        );
        glCompileShader(vertex_shader);
        check_shader_errors(vertex_shader, GL_COMPILE_STATUS);

        // Fragment shader
        const FRAG_SHADER_SRC: &str =
            "#version 330 core\nout vec4 FragColor;\n\nvoid main()\n{\n    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);\n}";
        let fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
        glShaderSource(
            fragment_shader,
            1,
            &(FRAG_SHADER_SRC.as_bytes().as_ptr().cast()),
            &(FRAG_SHADER_SRC.len().try_into()?),
        );
        glCompileShader(fragment_shader);
        check_shader_errors(fragment_shader, GL_COMPILE_STATUS);

        //Create shader program, link, check for errors
        let shader_program = glCreateProgram();
        glAttachShader(shader_program, vertex_shader);
        glAttachShader(shader_program, fragment_shader);
        glLinkProgram(shader_program);
        check_shader_errors(shader_program, GL_LINK_STATUS);

        // Final linked verion of our shaders. Every shader/render call after this function will use this shading program
        glUseProgram(shader_program);

        // Delete shaders after linking
        glDeleteShader(vertex_shader);
        glDeleteShader(fragment_shader);

        // Vertice input
        let triangle_verts = [
            0.5f32, 0.5f32, 0.0f32, 0.5f32, -0.5f32, 0.0f32, -0.5f32, -0.5f32, 0.0f32, -0.5f32,
            0.5f32, 0.0f32,
        ];
        let indices = [0, 1, 3, 1, 2, 3];

        // Create and initialize Buffer/Array objects
        // Vertex Buffer Objects (VBO) are where we manage batches of input data being sent to the GPU
        let mut vao = 0;
        let mut ebo = 0;
        let mut vbo = 0;
        glGenVertexArrays(1, &mut vao);
        glGenBuffers(1, &mut vbo);
        glGenBuffers(1, &mut ebo);

        // Bind VAO first, then bind and set vertex buffers, then configure vertex attribs
        // Binds assign our buffers a certain buffer type
        // Vertex Array Objects(VAO) can have attributes bound to them
        // Lets use configure vertex attributes once and use many times for drawing
        glBindVertexArray(vao);
        glBindBuffer(GL_ARRAY_BUFFER, vbo);
        glBufferData(
            GL_ARRAY_BUFFER, //
            size_of_val(&triangle_verts).try_into()?,
            triangle_verts.as_ptr().cast(),
            GL_STATIC_DRAW, // This hints our GPU how the data will be used
        );

        // Element Buffer Objects let use define shared indices for more efficient drawing instead of drawing each triangle with unique, individual vertices
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
        glBufferData(
            GL_ELEMENT_ARRAY_BUFFER,
            size_of_val(&indices).try_into()?,
            indices.as_ptr().cast(),
            GL_STATIC_DRAW,
        );

        // Tells OpenGL how to interpret our vertex attributes
        glVertexAttribPointer(
            0,                                  // Index of the attribute we want to change (layout = _)
            3, // Number of elements in the data we're sending. We're sending a vec3 so it has 3 elements.
            GL_FLOAT, // We're sending floats
            0, // Boolean value if we want the data to be normalized or not
            (size_of::<f32>() * 3).try_into()?, // Stride is how large our chunk of data is. We're sending 3 * f32.
            0 as *const _, // Offset is void* since the position of our data is at the start of the array
        );
        glEnableVertexAttribArray(0); // Enables our previously defined attribute at the given index

        // Unbind VBO
        glBindBuffer(GL_ARRAY_BUFFER, 0);

        // glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
    }

    event_loop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = glutin::event_loop::ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            unsafe {
                glClearColor(0.1, 0.1, 0.2, 1.0);
                glClear(GL_COLOR_BUFFER_BIT);
            }
            context.window().request_redraw();
        }
        Event::RedrawRequested(_) => unsafe {
            glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0 as *const _);
            context.swap_buffers().unwrap();
            println!("Render")
        },
        _ => {
            // catch-all
        }
    });
    // Ok(())
}

unsafe fn check_shader_errors(input: u32, status_enum: GLenum) {
    let mut success = 0;
    if status_enum == GL_COMPILE_STATUS {
        glGetShaderiv(input, status_enum, &mut success);
    } else if status_enum == GL_LINK_STATUS {
        glGetProgramiv(input, status_enum, &mut success)
    }
    if success == 0 {
        let mut v: Vec<u8> = Vec::with_capacity(1024);
        let mut log_len = 0_i32;
        glGetShaderInfoLog(input, 1024, &mut log_len, v.as_mut_ptr().cast());
        v.set_len(log_len.try_into().unwrap());
        panic!("Shader Compile Error: {}", String::from_utf8_lossy(&v));
    }
}
