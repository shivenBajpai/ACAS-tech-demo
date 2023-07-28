use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum StitchingError {
    #[error("Buffer is empty")]
    EmptyBufferError,
    #[error("Buffer is incomplete")]
    BufferSizeMismatch
}

pub struct ImageResult<T: Clone> (Result<(u32,u32,Vec<T>),StitchingError>);

pub fn rotate<T: Clone>(buf: &[T], empty: &[T], channels: u32, width: u32, height: u32, angle: f64) -> Result<(u32,u32,Vec<T>),StitchingError>
{
    if buf.is_empty() {
        return Err(StitchingError::EmptyBufferError)
    }
    if buf.len() != (width*height*channels) as usize {
        return Err(StitchingError::BufferSizeMismatch)
    }

    let sin = angle.sin();
    let cos = angle.cos();

    let widthf = width as f64;
    let heightf = height as f64;

    let diag_length = (widthf.powf(2.0)+heightf.powf(2.0)).sqrt();
    let diag_angle_with_horizontal = (heightf/widthf).atan();
    let diag_angle_1 = diag_angle_with_horizontal + angle;
    let diag_angle_2 = -diag_angle_with_horizontal + angle;

    let new_width = (f64::max(diag_angle_1.cos().abs(), diag_angle_2.cos().abs())*diag_length).round();
    let new_height = (f64::max(diag_angle_1.sin().abs(), diag_angle_2.sin().abs())*diag_length).round();

    //println!("diagonal length {} at angle of diagonals {} & {} gives dimensions {}x{}",diag_length,diag_angle_1,diag_angle_2,new_width,new_height);

    let mut result_buffer: Vec<T> = Vec::with_capacity((new_height*new_width) as usize*channels as usize);
    
    for y in (0..new_width as u32).map(|x| x as f64) {
        for x in (0..new_width as u32).map(|x| x as f64) {

            let pos: [f64; 2] = [x+0.5-(new_width)/2.0,y+0.5-(new_height)/2.0];

            //println!("Taking OutPix ({},{}) which has position vector {:?}",x,y,pos);
            let x_along_oldx: f64 = (pos[0]*cos - pos[1]*sin)+(widthf/2.0);

            /* if 0.0 < x_along_oldx && x_along_oldx < widthf {
                let y_along_oldy: f64 = (pos[0]*sin + pos[1]*cos)+(heightf/2.0);
                
                if 0.0 < y_along_oldy && y_along_oldy < heightf {
                    let index = ((y_along_oldy as u32*width+x_along_oldx as u32)*channels) as usize;
                    //println!("Pulling pixel {} which is {:?}",index,&buf[index..index+channels as usize]);
                    result_buffer.extend_from_slice(&buf[index..index+channels as usize]);
                    continue;
                }
            }

            result_buffer.extend_from_slice(empty); */

            let y_along_oldy: f64 = (pos[0]*sin + pos[1]*cos)+(heightf/2.0);
            //println!("Resolved vector along original axis ({},{})",x_along_oldx,y_along_oldy); // (-1.28,-4.781)

             if 0.0 < x_along_oldx && x_along_oldx < widthf && 0.0 < y_along_oldy && y_along_oldy < heightf {
                let index = ((y_along_oldy as u32*width+x_along_oldx as u32)*channels) as usize;
                //println!("Pulling pixel {} which is {:?}",index,&buf[index..index+channels as usize]);
                result_buffer.extend_from_slice(&buf[index..index+channels as usize])
            }
            else {
                result_buffer.extend_from_slice(empty);
            }

        }
    }

    Ok((new_width as u32,new_height as u32,result_buffer))
}

pub fn downscale<T: Clone>(buf: &[T], channels: u32, width: u32, height: u32, factor: f64) -> Result<(u32,u32,Vec<T>),StitchingError> 
{
    let new_width = (width as f64/factor).floor() as usize;
    let new_height = (height as f64/factor).floor() as usize;

    let mut result_buffer: Vec<T> = Vec::with_capacity((new_height*new_width)*channels as usize);

    for y in 0..new_height {
        for x in 0..new_width {
            let index = ((y as f64*factor).round()*width as f64+(x as f64*factor).round()) as usize*channels as usize;

            result_buffer.extend_from_slice(&buf[index..index+channels as usize])
        }
    }

    Ok((new_width as u32,new_height as u32,result_buffer))
}