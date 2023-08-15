use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
/// The Types of Errors that can occur when reading a buffer in rotation/stitching actions
pub enum ParsingError {
    #[error("Buffer is empty")]
    /// The image buffer passed contains no data
    EmptyBufferError,
    #[error("Buffer is incomplete")]
    /// The length image buffer passed does not match the dimensions given. 
    /// 
    /// length of buffer != width\*height\*channels
    BufferSizeMismatch
}

/// The Ordering for which image ends up on top
pub enum StitchingOrder {
    /// Keep the "source" (unrotated image) on top
    SourceOnTop,
    /// Keep the appendage (rotated image) on top
    AppendageOnTop
}

/// Determines the Algorithm used for rotation before stitching
pub enum StitchingQuality {
    /// Equivalent to fast_rotate()
    Fast,
    /// Equivalent to fancy_rotate()
    Fancy
}

/// A Trait that designates valid subpixel types for stitching operations
/// 
/// Already Implemented for all unsigned ints
pub trait StitchableType: Clone + Copy + TryFrom<i32> {
    /// Returns The Maximum value of this type
    fn maxvalue() -> Self;
}

impl StitchableType for u8 {
    fn maxvalue() -> u8 {
        std::u8::MAX
    }
}   

impl StitchableType for u16 {
    fn maxvalue() -> u16 {
        std::u16::MAX
    }
}   

impl StitchableType for u32 {
    fn maxvalue() -> u32 {
        std::u32::MAX
    }
}   

impl StitchableType for u64 {
    fn maxvalue() -> u64 {
        std::u64::MAX
    }
}

impl StitchableType for u128 {
    fn maxvalue() -> u128 {
        std::u128::MAX
    }
}

/// (Width, Height, Image buffer) or ParsingError
pub type StitchingResult<T> = Result<(usize,usize,Vec<T>),ParsingError>;

/// Rotates an image using the fancy algorithm.
/// Wont introduce any new colors as there is no color interpolation 
/// 
/// # Arguments
/// 
/// - buf - The image
/// - width, height - Dimensions of image
/// - channels - No. of channels per pixel
/// - empty - Empty space will be filled with this value
/// - angle - The angle of rotation (in radians), 
///   - positive => Anticlockwise, 
///   - negative => Clockwise
/// 
/// use fast_rotate() for faster rotation
pub fn fancy_rotate<T: Clone + std::fmt::Debug>(buf: &[T], empty: &[T], channels: usize, width: usize, height: usize, angle: f64) -> StitchingResult<T> where [T]: Eq + std::hash::Hash{
    
    let image2x = upscale(&buf, channels, width, height);
    let image4x = upscale(&image2x.2.as_slice(), channels,image2x.0, image2x.1);
    let image8x = upscale(&image4x.2.as_slice(), channels,image4x.0, image4x.1);

    let image_rotated = fast_rotate(image8x.2.as_slice(), empty, channels, image8x.0, image8x.1, angle)?;

    let downscaled = downscale(image_rotated.2.as_slice(), channels, image_rotated.0, image_rotated.1, 8);

    Ok(downscaled)
}

/// Rotates an image using the fast algorithm, result may be noisy for low resolution images. 
/// Wont introduce any new colors as there is no color interpolation 
/// 
/// # Arguments
/// 
/// - buf - The image
/// - width, height - Dimensions of image
/// - channels - No. of channels per pixel
/// - empty - Empty space will be filled with this value
/// - angle - The angle of rotation (in radians), 
///   - positive => Anticlockwise, 
///   - negative => Clockwise
/// 
/// use fancy_rotate() for higher quality rotation
pub fn fast_rotate<T: Clone + std::fmt::Debug>(buf: &[T], empty: &[T], channels: usize, width: usize, height: usize, angle: f64) -> StitchingResult<T> {
    if buf.is_empty() {
        return Err(ParsingError::EmptyBufferError)
    }
    if buf.len() != width*height*channels {
        return Err(ParsingError::BufferSizeMismatch)
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

    println!("diagonal length {} at angle of diagonals {} & {} gives dimensions {}x{}",diag_length,diag_angle_1,diag_angle_2,new_width,new_height);

    let mut result_buffer: Vec<T> = Vec::with_capacity((new_height*new_width) as usize*channels);
    
    for y in (0..new_height as u32).map(|x| x as f64) {
        for x in (0..new_width as u32).map(|x| x as f64) {

            let pos: [f64; 2] = [x+0.5-(new_width)/2.0,y+0.5-(new_height)/2.0];

            let x_along_oldx: f64 = ((pos[0]*cos - pos[1]*sin)+(widthf/2.0)-0.5).round();

            if 0.0 <= x_along_oldx && x_along_oldx < widthf {
                let y_along_oldy: f64 = ((pos[0]*sin + pos[1]*cos)+(heightf/2.0)-0.5).round();
                
                if 0.0 <= y_along_oldy && y_along_oldy < heightf {
                    let index = ((y_along_oldy*width as f64)+x_along_oldx) as usize*channels;
                    //println!("Filling pixel {},{} (vec: {:?} )from {},{} index {} which is {:?}",x,y,pos,(pos[0]*cos - pos[1]*sin)+(widthf/2.0),(pos[0]*sin + pos[1]*cos)+(heightf/2.0),index,&buf[index..index+channels as usize]);
                    result_buffer.extend_from_slice(&buf[index..index+channels]);
                    continue;
                }
            }
            //println!("Filling pixel {},{} with emptiness",x,y);
            result_buffer.extend_from_slice(empty); 

            /* let y_along_oldy: f64 = ((pos[0]*sin + pos[1]*cos)+(heightf/2.0)).round();
            println!("Resolved vector along original axis ({},{})",x_along_oldx,y_along_oldy); // (-1.28,-4.781)

            if 0.0 < x_along_oldx && x_along_oldx < widthf && 0.0 < y_along_oldy && y_along_oldy < heightf {
                let index = (y_along_oldy*width as f64+x_along_oldx as f64) as usize*channels;
                println!("Pulling pixel {} which is {:?}",index,&buf[index..index+channels as usize]);
                result_buffer.extend_from_slice(&buf[index..index+channels as usize])
            }
            else {
                result_buffer.extend_from_slice(empty);
            } */
        }
    }

    println!("Returning");
    Ok((new_width as usize,new_height as usize,result_buffer))
}

/// Stitches Two images together
/// 
/// # Arguments
/// 
/// - src, src_dimensions, src_anchor , src_angle - The source image, dimensions and point of stitching and desired angle of stitched appendage
/// - appendage, appendage_dimensions, appendage_anchor , appendage_angle - The appendage image, dimensions and point of stitching and its current angle in image
/// - empty - Equivalent of empty pixel
/// - channels - No. of channels per pixel
pub fn stitch<T>(src: &[T], appendage: &[T], empty: &[T], channels: usize, src_dimensions: (usize,usize), src_anchor: (usize,usize), src_angle: f64, appendage_dimensions: (usize,usize), appendage_anchor: (usize,usize), appendage_angle: f64, top: StitchingOrder, quality: StitchingQuality) -> StitchingResult<T> 
where T: StitchableType + std::fmt::Debug, f32: From<T>, [T]: Eq + std::hash::Hash
{
    let rotation = src_angle - appendage_angle;

    let rotated = {
        match quality {
            StitchingQuality::Fancy => fancy_rotate(appendage, empty, channels, appendage_dimensions.0, appendage_dimensions.1, rotation)?,
            StitchingQuality::Fast => fast_rotate(appendage, empty, channels, appendage_dimensions.0, appendage_dimensions.1, rotation)?
        }
    };
    let rotated_anchor_pos = rotate_point(appendage_anchor, appendage_dimensions.0, appendage_dimensions.1, rotation);

    // T, R, B, L
    let src_dist: [usize; 4] = [
        src_anchor.1,
        src_dimensions.0-src_anchor.0,
        src_dimensions.1-src_anchor.1,
        src_anchor.0
    ];

    let rot_dist: [usize; 4] = [
        rotated_anchor_pos.1,
        rotated.0-rotated_anchor_pos.0,
        rotated.1-rotated_anchor_pos.1,
        rotated_anchor_pos.0
    ];

    let dist: Vec<usize> = src_dist.iter().zip(rot_dist.iter()).map(|(x, y)| usize::max(*x,*y)).collect();
    
    println!("Post rotation anchor is at {},{} in an image of {},{}",rotated_anchor_pos.0,rotated_anchor_pos.1,rotated.0,rotated.1);

    let width = dist[1] + dist[3];
    let height = dist[0] + dist[2];

    let rotated_image_topleft = (dist[3]-rotated_anchor_pos.0,dist[0]-rotated_anchor_pos.1);
    let src_image_topleft = (dist[3]-src_anchor.0,dist[0]-src_anchor.1);

    let mut res = empty.to_vec().repeat(height*width*channels);

    println!("Toplefts are at {},{} and {},{}",rotated_image_topleft.0,rotated_image_topleft.1,src_image_topleft.0,src_image_topleft.1);

    for y in 0..src_dimensions.1 {
        for x in 0..src_dimensions.0 {
            for c in 0..channels {
                res[(src_image_topleft.1+y)*width*channels+(src_image_topleft.0+x)*channels+c] = src[y*src_dimensions.0*channels+x*channels+c];
            }
        }
    }

    match top{
        StitchingOrder::AppendageOnTop => {
            for y in 0..rotated.1 {
                for x in 0..rotated.0 {
                    //println!("Pulled pixel {:?} for {},{} Had index {}",&rotated.2[y*rotated.0*channels+x*channels..y*rotated.0*channels+x*channels+channels],x,y,y*rotated.0*channels+x*channels);
                    let blended = blend(&rotated.2[y*rotated.0*channels+x*channels..y*rotated.0*channels+x*channels+channels], &res[(rotated_image_topleft.1+y)*width*channels+(rotated_image_topleft.0+x)*channels..(rotated_image_topleft.1+y)*width*channels+(rotated_image_topleft.0+x)*channels+channels]);
                    for c in 0..channels {
                        res[(rotated_image_topleft.1+y)*width*channels+(rotated_image_topleft.0+x)*channels+c] = blended[c]
                    }
                }
            }
        },
        StitchingOrder::SourceOnTop => {
            for y in 0..rotated.1 {
                for x in 0..rotated.0 {
                    let blended = blend(&res[(rotated_image_topleft.1+y)*width*channels+(rotated_image_topleft.0+x)*channels..(rotated_image_topleft.1+y)*width*channels+(rotated_image_topleft.0+x)*channels+channels], &rotated.2[y*rotated.0*channels+x*channels..y*rotated.0*channels+x*channels+channels]);
                    for c in 0..channels {
                        res[(rotated_image_topleft.1+y)*width*channels+(rotated_image_topleft.0+x)*channels+c] = blended[c]
                    }
                }
            }
        }
    };
        

    Ok((width,height,res))
}

fn rotate_point(point: (usize,usize), width: usize, height: usize, angle: f64) -> (usize,usize) {
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
    
    println!("Rotate Point:  dims {}x{}",new_width,new_height);

    let pos_vector = (
        point.0 as f64 - widthf/2.0 + 0.5,
        point.1 as f64 - heightf/2.0 + 0.5,
    );

    let point_rotated_wrt_center = (
        pos_vector.0*cos + pos_vector.1*sin,
        pos_vector.1*cos - pos_vector.0*sin
    );

    let final_point = (
        (point_rotated_wrt_center.0 + new_width/2.0) as usize,
        (point_rotated_wrt_center.1 + new_height/2.0) as usize
    );

    println!("Rotate Point: vec {:?} rotated to {:?} resolved as {:?}",pos_vector,point_rotated_wrt_center,final_point);

    final_point
}

fn blend<T>(top: &[T], bottom: &[T]) -> Vec<T>
where T: Copy + Clone + TryFrom<i32> + StitchableType, f32: From<T>
{
    let alpha= f32::from(top.last().expect("Pixel cannot have 0 channels").clone())/f32::from(T::maxvalue());
    let alphacomp = 1.0-alpha;
    let finalalpha = if alpha > 0.5 { top[top.len()-1] } else { bottom[top.len()-1] };
    let mut res: Vec<T> = Vec::with_capacity(top.len());

    for i in 0..top.len()-1 {
        let average: f32 = ( f32::from(top[i].clone())* alpha + f32::from(bottom[i].clone()) * alphacomp).round();
        let intaverage: i32 = average as i32;
        res.push(T::try_from(intaverage).unwrap_or(T::maxvalue()))
    }

    res.push(finalalpha);

    res
}

fn downscale<T: Clone>(buf: &[T], channels: usize, width: usize, height: usize, factor: usize) -> (usize,usize,Vec<T>) where [T]: Eq + std::hash::Hash + std::fmt::Debug {
    let new_width = (width as f64/factor as f64).floor() as usize;
    let new_height = (height as f64/factor as f64).floor() as usize;

    let mut result_buffer: Vec<T> = Vec::with_capacity((new_height*new_width)*channels as usize);

    for y in 0..new_height {
        // let y_component = y*factor*width;

        for x in 0..new_width {
            // let index = y_component + x*factor*channels;

            result_buffer.extend_from_slice(&find_mode(buf, channels, width, x*factor, y*factor, factor))
        }
    }

    (new_width,new_height,result_buffer)
}

fn find_mode<T>(buf: &[T], channels: usize, width: usize, startx: usize, starty: usize, block_size: usize) -> &[T] where [T]: Eq + std::hash::Hash + std::fmt::Debug {

    println!("Called with {}, {}, {}, {}",width,startx,starty,block_size);

    let mut count: std::collections::HashMap<&[T], u8> = HashMap::new();
    let mut max_val: &[T] = &buf[0..channels];
    let mut max_count: u8 = 0;

    for x in startx..startx+block_size {
        for y in starty..starty+block_size {

            let index = (y*width+x)*channels;
            *count.entry(&buf[index..index+channels]).or_insert(0) += 1;
        }
    }

    println!("Counted {:?}",count);

    for (key,value) in count.iter() {
        if *value > max_count {
            max_count = *value;
            max_val = key;
        }
    }

    println!("Returning {:?}",max_val);

    return max_val;
}

fn upscale<T>(buf: &[T],channels: usize,width: usize,height: usize) -> (usize,usize,Vec<T>)
where T: Clone + std::fmt::Debug, [T]: Eq
{
    let new_width: usize = width*2 as usize;
    let new_height: usize = height*2 as usize;

    let row_offset = width*channels;

    let mut scaled = vec![buf[0].clone(); new_width * new_height * channels];

    // Apply the algorithm to the center
    for y in 1..height as usize - 1 {
        let source_y_offset = y * row_offset;
        let scaled_y_offset = y * 2 * new_width * channels;

        let up_offset = source_y_offset - row_offset;
        let down_offset = source_y_offset + row_offset;

        for x in 1..width as usize - 1 {
            let pos = source_y_offset + x * channels;
            apply_scale2x_block(
                &mut scaled,
                scaled_y_offset + x * 2 * channels,
                channels,
                new_width,
                (
                    // Center
                    &buf[pos..pos+channels],
                    // Up
                    &buf[pos-row_offset..pos-row_offset+channels],
                    // Left
                    &buf[pos-channels..pos],
                    // Down
                    &buf[pos+row_offset..pos+row_offset+channels],
                    // Right
                    &buf[pos+channels..pos+2*channels],
                ),
            );
        }

        // Left most column
        let p = &buf[source_y_offset..source_y_offset+channels];
        apply_scale2x_block(
            &mut scaled,
            scaled_y_offset,
            channels,
            new_width,
            (p, &buf[up_offset..up_offset+channels], p, &buf[down_offset..down_offset+channels], &buf[source_y_offset+channels..source_y_offset+2*channels]),
        );

        // Right most column
        let index = source_y_offset+row_offset-channels;
        let p = &buf[index..index+channels];
        apply_scale2x_block(
            &mut scaled,
            scaled_y_offset + (new_width - 2)*channels,
            channels,
            new_width,
            (p, &buf[source_y_offset-channels..source_y_offset], &buf[index-channels..index], &buf[index+row_offset..index+row_offset+channels], p),
        );
    }

    println!("Now Doing Top and Bottom");

    for x in 1..width - 1 {
        // Apply the algorithm to the first row
        let x_offset = x*channels; 
        let p = &buf[x_offset..x_offset+channels];
        apply_scale2x_block(
            &mut scaled,
            x*channels*2,
            channels,
            new_width,
            (p, p, &buf[x_offset-channels..x_offset], &buf[x_offset+row_offset..x_offset+row_offset+channels], &buf[x_offset+channels..x_offset+2*channels]),
        );

        // Apply the algorithm to the last row
        let index = (height - 1) * width * channels + x_offset;
        let p = &buf[index..index+channels];
        let scaled_y_this = ((height - 1) * 2) * new_width * channels;
        apply_scale2x_block(
            &mut scaled,
            scaled_y_this + x*channels*2,
            channels,
            new_width,
            (p, &buf[index-row_offset..index-row_offset+channels], &buf[index-channels..index], p, &buf[index+channels..index+2*channels]),
        );
    }

    // Apply the algorithms to the corners
    println!("Now Doing Corners");

    // Top left corner
    let p = &buf[0..channels];
    apply_scale2x_block(&mut scaled, 0, channels, new_width, (p, p, p, &buf[row_offset..row_offset+channels], &buf[channels..2*channels]));

    // Top right corner
    let x_right = width - 1;
    let p = &buf[row_offset-channels..row_offset];
    apply_scale2x_block(
        &mut scaled,
        (new_width-2)*channels,
        channels,
        new_width,
        (p, p, &buf[row_offset-2*channels..row_offset-channels], &buf[row_offset*2-channels..row_offset*2], p),
    );

    // Bottom left corner
    let y_bottom = (height-1)*width*channels;
    let p = &buf[y_bottom..y_bottom+channels];
    apply_scale2x_block(
        &mut scaled,
        (new_height - 2)*new_width*channels,
        channels,
        new_width,
        (p, &buf[y_bottom-row_offset..y_bottom-row_offset+channels], p, p, &buf[y_bottom+channels..y_bottom+2*channels]),
    );

    // Bottom right corner
    let y_bottom_right = y_bottom + x_right;
    let p = &buf[y_bottom_right..y_bottom_right+channels];
    apply_scale2x_block(
        &mut scaled,
        ((new_height-2)*new_width+new_width-2)*channels,
        channels,
        new_width,
        (p, &buf[y_bottom_right-row_offset..y_bottom_right-row_offset+channels], &buf[y_bottom_right-channels..y_bottom_right], p, p)
    );

    (new_width,new_height,scaled)
}


fn apply_scale2x_block<P>(scaled: &mut Vec<P>, pos: usize, channels: usize, width: usize, pixels: (&[P], &[P], &[P], &[P], &[P]))
where P: Clone + std::fmt::Debug, [P]: Eq
{   
    copy_to_vec(scaled, pos, if pixels.2 == pixels.1 && pixels.2 != pixels.3 && pixels.1 != pixels.4 { pixels.1 } else { pixels.0 });
    copy_to_vec(scaled, pos+channels, if pixels.1 == pixels.4 && pixels.1 != pixels.2 && pixels.4 != pixels.3 { pixels.4 } else { pixels.0 });
    copy_to_vec(scaled, pos+width*channels, if pixels.3 == pixels.2 && pixels.3 != pixels.4 && pixels.2 != pixels.1 { pixels.2 } else { pixels.0 });
    copy_to_vec(scaled, pos+(width+1)*channels, if pixels.4 == pixels.3 && pixels.4 != pixels.1 && pixels.3 != pixels.2 { pixels.3 } else { pixels.0 });
    // scaled[pos..pos+channels] = (if pixels.2 == pixels.1 && pixels.2 != pixels.3 && pixels.1 != pixels.4 { pixels.1 } else { pixels.0 });
    // scaled[pos+1..pos+1+channels] = (if pixels.1 == pixels.4 && pixels.1 != pixels.2 && pixels.4 != pixels.3 { pixels.4 } else { pixels.0 }).clone();
    // scaled[pos+width..pos+width+channels] = (if pixels.3 == pixels.2 && pixels.3 != pixels.4 && pixels.2 != pixels.1 { pixels.2 } else { pixels.0 }).clone();
    // scaled[pos+width+1..pos+width+1+channels] = (if pixels.4 == pixels.3 && pixels.4 != pixels.1 && pixels.3 != pixels.2 { pixels.3 } else { pixels.0 }).clone();
}

fn copy_to_vec<P: Clone + std::fmt::Debug>(destination: &mut Vec<P>, start_index: usize, source: &[P]) {
    println!("Called with {:?} to put at index {} on a vec of len {}",source,start_index,destination.len());
    for (i,item) in source.iter().enumerate() {
        destination[start_index+i] = item.clone()
    }
}