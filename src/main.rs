mod c_macros;

use cgmath::{vec3, Deg, InnerSpace, Matrix, Matrix4, Point3};
use gl33::{global_loader::*, *};
use glutin::{
    dpi::{LogicalPosition, Position},
    event::{Event, WindowEvent, KeyboardInput},
    event_loop::EventLoop,
    window::WindowBuilder,
    Api,
};
use std::ffi::c_void;
use std::mem::{size_of, size_of_val};

// Screen Settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

// Broken Enums
const _GL_NEAREST: i32 = 0x2600;
const _GL_LINEAR: i32 = 0x2601;
const _GL_NEAREST_MIPMAP_LINEAR: i32 = 0x2702;
const _GL_RGB: i32 = 0x1907;
const _GL_FALSE: u8 = 0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build Window, Context, Event Loop
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("Lyra-OpenGL3.3")
        .with_inner_size(glutin::dpi::LogicalSize::new(SCR_WIDTH, SCR_HEIGHT))
        .with_position(Position::Logical(LogicalPosition::new(1600.0, 200.0)));
    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_vsync(true)
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    let context = unsafe { context.make_current().unwrap() };
    println!("Pixel fomrat of the window's GL context: {:?}", context.get_pixel_format());

    // Load in all our OpenGL Functions
    unsafe {
        load_global_gl(&|ptr| {
            let c_str = std::ffi::CStr::from_ptr(ptr as *const i8);
            let r_str = c_str.to_str().unwrap();
            context.get_proc_address(r_str) as _
        })
    };

    // Vertex shader | TODO: Improve file reading later
    let vert_shader_src: String = std::fs::read_to_string("./shaders/shader.vert")?;
    let vertex_shader = glCreateShader(GL_VERTEX_SHADER);
    unsafe {
        glShaderSource(
            vertex_shader,
            1,
            &(vert_shader_src.as_str().as_bytes().as_ptr().cast()),
            &(vert_shader_src.as_str().len().try_into()?),
        )
    };
    glCompileShader(vertex_shader);
    unsafe { check_shader_errors(vertex_shader, GL_COMPILE_STATUS) };

    // Fragment shader
    let frag_shader_src: String = std::fs::read_to_string("./shaders/shader.frag")?;
    let fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);

    unsafe {
        glShaderSource(
            fragment_shader,
            1,
            &(frag_shader_src.as_bytes().as_ptr().cast()),
            &(frag_shader_src.len().try_into()?),
        )
    };
    glCompileShader(fragment_shader);
    unsafe { check_shader_errors(fragment_shader, GL_COMPILE_STATUS) };

    //Create shader program, link, check for errors
    let shader_program = glCreateProgram();
    glAttachShader(shader_program, vertex_shader);
    glAttachShader(shader_program, fragment_shader);
    glLinkProgram(shader_program);
    unsafe { check_shader_errors(shader_program, GL_LINK_STATUS) };

    // Final linked verion of our shaders. Every shader/render call after this function will use this shading program
    glUseProgram(shader_program);

    // Delete shaders after linking
    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);

    // Vertice input
    let vertices = [
        -0.5f32, -0.5, -0.5, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5, 0.5, -0.5, 1.0, 1.0, -0.5, 0.5, -0.5, 0.0, 1.0, -0.5, -0.5,
        -0.5, 0.0, 0.0, -0.5, -0.5, 0.5, 0.0, 0.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 1.0, -0.5, 0.5, 0.5, 0.0, 1.0, -0.5,
        -0.5, 0.5, 0.0, 0.0, -0.5, 0.5, 0.5, 1.0, 0.0, -0.5, 0.5, -0.5, 1.0, 1.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, -0.5, 0.5, 0.0,
        0.0, -0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, 0.5,
        0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, -0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, -0.5, 1.0, 1.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.5, -0.5, 0.5, 1.0, 0.0, -0.5,
        -0.5, 0.5, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, 0.5, -0.5, 0.0, 1.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0,
        -0.5, 0.5, 0.5, 0.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0,
    ];

    // Create and initialize Buffer/Array objects
    // Vertex Buffer Objects (VBO) are where we manage batches of input data being sent to the GPU
    let (mut vbo, mut vao) = (0, 0);
    unsafe {
        glGenVertexArrays(1, &mut vao);
        glGenBuffers(1, &mut vbo);

        // Bind VAO first, then bind and set vertex buffers, then configure vertex attribs
        // Binds assign our buffers a certain buffer type
        // Vertex Array Objects(VAO) can have attributes bound to them
        // Lets use configure vertex attributes once and use many times for drawing
        glBindVertexArray(vao);
        glBindBuffer(GL_ARRAY_BUFFER, vbo);
        glBufferData(
            GL_ARRAY_BUFFER, //
            size_of_val(&vertices).try_into()?,
            vertices.as_ptr().cast(),
            GL_STATIC_DRAW, // This hints our GPU how the data will be used
        );

        // Tells OpenGL how to interpret our vertex attributes
        // Position attribute
        glVertexAttribPointer(
            0,                                  // Index of the attribute we want to change (layout = _)
            3,                                  // Number of elements in the data we're sending. We're sending a vec3 so it has 3 elements.
            GL_FLOAT,                           // We're sending floats
            0,                                  // Boolean value if we want the data to be normalized or not
            (size_of::<f32>() * 5).try_into()?, // Stride is how large our chunk of data is. We're sending 3 * f32.
            0 as *const c_void,                 // Offset is void* since the position of our data is at the start of the array
        );
        glEnableVertexAttribArray(0);

        // TexCoords
        glVertexAttribPointer(1, 2, GL_FLOAT, 0, (size_of::<f32>() * 5).try_into()?, (3 * size_of::<f32>()) as *const c_void);
        glEnableVertexAttribArray(1);

        // Unbind VBO
        glBindBuffer(GL_ARRAY_BUFFER, 0);

        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, _GL_NEAREST_MIPMAP_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, _GL_LINEAR);
    }

    // Texture
    let my_tex = image::open("./resources/alfie.jpg").expect("Failed to load texture").flipv();
    let my_tex_data = my_tex.to_rgb8();

    let mut texture: u32 = 0;
    unsafe {
        glGenTextures(1, &mut texture);
        glBindTexture(GL_TEXTURE_2D, texture);
        glTexImage2D(
            GL_TEXTURE_2D,
            0,
            _GL_RGB,
            my_tex.width() as i32,
            my_tex.height() as i32,
            0,
            GL_RGB,
            GL_UNSIGNED_BYTE,
            (&my_tex_data as &[u8]).as_ptr() as *const c_void,
        );
        glGenerateMipmap(GL_TEXTURE_2D);
    }

    let start_time = std::time::Instant::now();

    let mesh_positions = [
        vec3(0.0f32, 0.0, 0.0),
        vec3(2.0, 5.0, -15.0),
        vec3(-1.5, -2.2, -2.5),
        vec3(-3.8, -2.0, -12.3),
        vec3(2.4, -0.4, -3.5),
        vec3(-1.7, 3.0, -7.5),
        vec3(1.3, -2.0, -2.5),
        vec3(1.5, 2.0, -2.5),
        vec3(1.5, 0.2, -1.5),
        vec3(-1.3, 1.0, -1.5),
    ];

    // Camera / Projection
    let up = vec3(0.0f32, 1.0, 0.0);
    let mut camera_pos = Point3 { x: 0.0f32, y: 0.0, z: 5.0 };
    let camera_target = Point3 { x: 0.0f32, y: 0.0, z: 0.0 };
    let camera_direction = camera_pos - camera_target;
    let camera_front = vec3(0.0f32, 0.0, -1.0);
    let camera_right = (up.cross(camera_direction)).normalize();
    let camera_up = camera_direction.cross(camera_right);

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
                glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

                let time = std::time::Instant::elapsed(&start_time).as_secs_f32();

                // let view = Matrix4::look_at_rh(camera_pos, camera_target, camera_up);
                let view = Matrix4::look_at_rh(camera_pos, camera_pos + camera_front, camera_up);

                let projection = cgmath::perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
                // note: currently we set the projection matrix each frame, but since the projection matrix rarely changes it's often best practice to set it outside the main loop only once.

                let model_loc = glGetUniformLocation(shader_program, c_str_ptr!("model"));
                let view_loc = glGetUniformLocation(shader_program, c_str_ptr!("view"));
                let projection_loc = glGetUniformLocation(shader_program, c_str_ptr!("projection"));

                glUniformMatrix4fv(view_loc, 1, _GL_FALSE, view.as_ptr());
                glUniformMatrix4fv(projection_loc, 1, _GL_FALSE, projection.as_ptr());

                glEnable(GL_DEPTH_TEST);
                gl_check_error();

                for (i, position) in mesh_positions.iter().enumerate() {
                    let mut model = Matrix4::from_translation(*position) * Matrix4::from_axis_angle(vec3(0.5, 1.0, 0.0).normalize(), cgmath::Rad(time));
                    let angle = 20.0 * i as f32;
                    model = model * Matrix4::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), Deg(angle));
                    glUniformMatrix4fv(model_loc, 1, _GL_FALSE, model.as_ptr());
                    glDrawArrays(GL_TRIANGLES, 0, 36);
                }
            }
            context.window().request_redraw();
        }
        Event::RedrawRequested(_) => {
            context.swap_buffers().unwrap();
        }
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

unsafe fn gl_check_error() {
    let error = glGetError();
    debug_assert!(error == GLenum(0x0), "OpenGL Error: {}", error.0)
}
