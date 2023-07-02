use crate::utils::manual_types::sdl2::SDL_Window;


pub fn SDL_GetWindowSize_safe(window: *const SDL_Window) -> (i32, i32)  {
    let mut outputX: i32 = 0;
    let mut outputY: i32 = 0;
    unsafe {
        super::sdl2::SDL_GetWindowSize(window, (&mut outputX), (&mut outputY));
    }
    (outputX, outputY)
}