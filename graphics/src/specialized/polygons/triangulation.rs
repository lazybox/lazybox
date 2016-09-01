//! Methods for converting shapes into triangles, inspired from piston graphics.

use std::f32;
use cgmath::{Point2, Vector2, Angle, Rad};

use super::Triangle;

pub fn stream_polygon<I, S>(mut points: I, mut stream: S)
    where I: Iterator<Item=[f32; 2]>, S: FnMut(&[Triangle])
{
    let mut triangles: [Triangle; 120] = [[[0.0; 2]; 3]; 120];

    let first = match points.next() { None => return, Some(p) => p };
    let mut g = match points.next() { None => return, Some(p) => p };
    let mut index = 0;
    for p in points {
        triangles[index] = [first, g, p];
        g = p;

        index += 1;
        if index >= triangles.len() {
            stream(&triangles[0..index]);
            index = 0;
        }
    }

    if index > 0 {
        stream(&triangles[0..index]);
    }
}

pub fn stream_ellipse<S>(center: Point2<f32>,
                         size: Vector2<f32>,
                         resolution: u32,
                         stream: S)
    where S: FnMut(&[Triangle])
{
    let cw = size.x * 0.5;
    let ch = size.y * 0.5;
    let mut i = 0;

    let points = FnIterator(|| {
        if i >= resolution { return None; }

        let angle = i as f32 / resolution as f32 * 2. * f32::consts::PI;
        i += 1;
        Some([center.x + angle.cos() * cw, center.y + angle.sin() * ch])
    });

    stream_polygon(points, stream);
}

pub fn stream_round_borders_line<S>(start: Point2<f32>, 
                                    end: Point2<f32>,
                                    cap_resolution: u32,
                                    radius: f32,
                                    stream: S)
    where S: FnMut(&[Triangle])
{
    let resolution = cap_resolution * 2;

    let diff = end - start;
    let half_pi = f32::consts::PI/2.0;
    let start_angle = Rad::atan2(diff.y, diff.x).0 + half_pi;

    let mut i = 0;
    
    let points = FnIterator(|| {
        let j = i;
        i += 1;

        if j >= resolution { 
            None 
        } else if j < cap_resolution {
            let ratio = j as f32 / (cap_resolution - 1) as f32;
            let angle = start_angle + ratio * f32::consts::PI;
            Some([start.x + angle.cos()*radius, start.y + angle.sin()*radius])
        } else {
            let ratio = (j - cap_resolution) as f32 / (cap_resolution - 1) as f32;
            let angle = start_angle + f32::consts::PI + ratio * f32::consts::PI;
            Some([end.x + angle.cos()*radius, end.y + angle.sin()*radius])
        }
    });
    stream_polygon(points, stream)
}

struct FnIterator<F: FnMut() -> Option<I>, I>(F);

impl<F: FnMut() -> Option<I>, I> Iterator for FnIterator<F, I> {
    type Item = I;

    fn next(&mut self) -> Option<I> { self.0() }
}