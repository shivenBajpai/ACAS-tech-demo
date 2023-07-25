use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum StitchingError {
    #[error("Buffer is empty")]
    EmptyBufferError,
    #[error("Buffer is incomplete")]
    BufferSizeMismatch
}


pub fn rotate<T>(buf: &[T], empty: &[T], channels: u64, width: u64, height: u64, angle: f64) -> Result<(u32,u32,Vec<T>),StitchingError>
where T: Clone + Copy + std::fmt::Debug
{
    if buf.is_empty() {
        return Err(StitchingError::EmptyBufferError)
    }
    if buf.len() != (width*height*channels) as usize {
        return Err(StitchingError::BufferSizeMismatch)
    }

    let sin = angle.sin();
    let cos = angle.cos();

    let diag_length = ((width.pow(2)+height.pow(2)) as f64).sqrt();
    let diag_angle_with_horizontal = (height as f64/width as f64).atan();
    let diag_angle_1 = diag_angle_with_horizontal + angle;
    let diag_angle_2 = -diag_angle_with_horizontal + angle;

    let new_width = (f64::max(diag_angle_1.cos().abs(), diag_angle_2.cos().abs())*diag_length).round();
    let new_height = (f64::max(diag_angle_1.sin().abs(), diag_angle_2.sin().abs())*diag_length).round();

    println!("diagonal length {} at angle of diagonals {} & {} gives dimensions {}x{}",diag_length,diag_angle_1,diag_angle_2,new_width,new_height);

    //let result_buffer: Vec<T> = empty.repeat(new_width*new_height);
    let mut result_buffer: Vec<T> = Vec::with_capacity((new_height*new_width) as usize*channels as usize);
    
    for y in (0..new_width as u32).map(|x| x as f64) {
        for x in (0..new_width as u32).map(|x| x as f64) {

            let pos: [f64; 2] = [x+0.5f64-(new_width)/2f64,y+0.5f64-(new_height)/2f64];

            println!("Taking OutPix ({},{}) which has position vector {:?}",x,y,pos);
            let x_along_oldx: f64 = (pos[0]*cos - pos[1]*sin)+(width as f64/2f64); // width as f64/2f64 + 
            let y_along_oldy: f64 = (pos[0]*sin + pos[1]*cos)+(height as f64/2f64); // height as f64/2f64 + 
            println!("Resolved vector along original axis ({},{})",x_along_oldx,y_along_oldy); // (-1.28,-4.781)

             if 0f64 < x_along_oldx && x_along_oldx < width as f64 && 0f64 < y_along_oldy && y_along_oldy < height as f64 {
                let index = ((y_along_oldy as u64*width+x_along_oldx as u64)*channels) as usize;
                println!("Pulling pixel {} which is {:?}",index,&buf[index..index+channels as usize]);
                result_buffer.extend_from_slice(&buf[index..index+channels as usize]) //index..index+channels
            }
            else {
                result_buffer.extend_from_slice(empty);
            } 
        }
    }

    Ok((new_width as u32,new_height as u32,result_buffer))
}