use std::cell::RefCell;
use std::rc::Rc;

use crate::math::Vector2;
use notan::draw::*;
use notan::prelude::{Color, Mouse};

pub mod math;

const GRAVITY: Vector2 = Vector2 { x: 0.0, y: 981.0 };
const CURSOR_RADIUS: f64 = 10.0;

pub trait Render {
    fn render(&self, draw: &mut Draw);
}

pub struct Cloth {
    points: Vec<Rc<RefCell<Point>>>,
    sticks: Vec<Rc<RefCell<Stick>>>,
    drag: f64,
    elasticity: f64,
}

impl Cloth {
    pub fn new(
        width: i32,
        height: i32,
        spacing: i32,
        start_x: i32,
        start_y: i32,
        elasticity: f64,
    ) -> Self {
        let mut points = Vec::new();
        let mut sticks = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let point = Rc::new(RefCell::new(Point::new(Vector2::new(
                    (start_x + x * spacing) as f64,
                    (start_y + y * spacing) as f64,
                ))));

                // Provided that the point is not the first point in the row, create a stick to the left
                if x != 0 {
                    let left_point = points.last().unwrap();
                    let stick = Rc::new(RefCell::new(Stick::new(
                        Rc::clone(&point),
                        Rc::clone(left_point),
                        spacing as f64,
                        elasticity,
                    )));
                    left_point.borrow_mut().add_stick(Rc::clone(&stick), 0);
                    point.borrow_mut().add_stick(Rc::clone(&stick), 0);

                    sticks.push(stick);
                }

                // Provided that the point is not the first point in the column, create a stick to the top
                if y != 0 {
                    let up_point = points.get((x + (y - 1) * width) as usize).unwrap();
                    let stick = Rc::new(RefCell::new(Stick::new(
                        Rc::clone(&point),
                        Rc::clone(up_point),
                        spacing as f64,
                        elasticity,
                    )));

                    up_point.borrow_mut().add_stick(Rc::clone(&stick), 1);
                    point.borrow_mut().add_stick(Rc::clone(&stick), 1);

                    sticks.push(stick);
                }

                // Pin half of the top points so that the cloth doesn't fall off the screen
                if y == 0 && x % 2 == 0 {
                    point.borrow_mut().pin();
                }

                points.push(point);
            }
        }
        Cloth {
            points,
            sticks,
            drag: 0.05,
            elasticity,
        }
    }

    pub fn update(&mut self, dt: f64, mouse: &Mouse, prev_mouse_position: Vector2) {
        for point in &self.points {
            let mut point = point.borrow_mut();

            // Check if the point is within the mouse's selection radius
            // Uses the square of the magnitude instead of distance since sqrt is expensive
            let dist_sq = (point.position - Vector2::from(mouse.position())).magnitude_squared();
            let selected = dist_sq <= CURSOR_RADIUS * CURSOR_RADIUS;

            let mut force = GRAVITY;

            // Apply force from mouse dragging
            if selected {
                if mouse.left_is_down() {
                    let diff = Vector2::from(mouse.position()) - prev_mouse_position;
                    let clamped = Vector2::new(
                        diff.x.clamp(-self.elasticity, self.elasticity),
                        diff.y.clamp(-self.elasticity, self.elasticity),
                    );
                    force += clamped * 10000.0;
                } else if mouse.right_is_down() {
                    point.break_sticks();
                }
            }

            point.update(dt, self.drag, force, selected);
        }

        // Apply stick constraints and remove broken sticks
        let mut to_remove = Vec::new();
        for (i, stick) in self.sticks.iter().enumerate() {
            let mut stick = stick.borrow_mut();
            if stick.broken {
                to_remove.push(i);
            }

            stick.update();
        }
        self.remove_sticks(to_remove);
    }

    fn remove_sticks(&mut self, indices: Vec<usize>) {
        for i in indices.iter().rev() {
            self.sticks.remove(*i);
        }
    }

    pub fn draw(&self, draw: &mut Draw) {
        for stick in &self.sticks {
            stick.borrow().render(draw);
        }
    }
}

struct Point {
    position: Vector2,
    prev_position: Vector2,
    initial_position: Vector2,
    sticks: [Option<Rc<RefCell<Stick>>>; 2],
    pinned: bool,
}

impl Point {
    fn new(position: Vector2) -> Self {
        Point {
            position,
            prev_position: position,
            initial_position: position,
            sticks: [None, None],
            pinned: false,
        }
    }

    fn break_sticks(&mut self) {
        if let Some(stick) = &self.sticks[0] {
            stick.borrow_mut().broken = true;
            self.sticks[0] = None;
        }
        if let Some(stick) = &self.sticks[1] {
            stick.borrow_mut().broken = true;
            self.sticks[1] = None;
        }
    }

    fn add_stick(&mut self, stick: Rc<RefCell<Stick>>, add_index: usize) {
        self.sticks[add_index] = Some(stick);
    }

    fn pin(&mut self) {
        self.pinned = true;
    }

    fn update(&mut self, dt: f64, drag: f64, acceleration: Vector2, selected: bool) {
        // Highlight
        for stick in self.sticks.iter().flatten() {
            stick.borrow_mut().selected = selected;
        }
        if self.pinned {
            self.position = self.initial_position;
            return;
        }

        // Solve for new position using verlet integration
        let new_position = self.position
            + (self.position - self.prev_position) * (1.0 - drag)
            + acceleration * (1.0 - drag) * dt * dt;
        self.prev_position = self.position;
        self.position = new_position;
    }
}

impl Render for Point {
    fn render(&self, draw: &mut Draw) {
        draw.circle(1.0)
            .position(self.position.x as f32, self.position.y as f32);
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::new(Vector2::ZERO)
    }
}

struct Stick {
    p1: Rc<RefCell<Point>>,
    p2: Rc<RefCell<Point>>,
    length: f64,
    elasticity: f64,
    selected: bool,
    broken: bool,
}

impl Stick {
    fn new(p1: Rc<RefCell<Point>>, p2: Rc<RefCell<Point>>, length: f64, elasticity: f64) -> Self {
        // Elasticity should be greater than 0 since its the percent of the length
        // that the stick can stretch before breaking.
        assert!(elasticity >= 0.0);
        Stick {
            p1,
            p2,
            length,
            elasticity,
            selected: false,
            broken: false,
        }
    }

    fn update(&mut self) {
        let offset = {
            let p1 = self.p1.borrow();
            let p2 = self.p2.borrow();

            let diff = p1.position - p2.position;
            let dist = diff.magnitude();

            // Break the stick if it stretches too much
            if dist > self.length * (1.0 + self.elasticity) {
                self.broken = true;
            }

            let diff_factor = (self.length - dist) / dist;
            diff * diff_factor * 0.5
        };

        let mut p1 = self.p1.borrow_mut();
        let mut p2 = self.p2.borrow_mut();
        p1.position += offset;
        p2.position -= offset;
    }
}

impl Render for Stick {
    fn render(&self, draw: &mut Draw) {
        let p1 = self.p1.borrow();
        let p2 = self.p2.borrow();
        draw.line(
            (p1.position.x as f32, p1.position.y as f32),
            (p2.position.x as f32, p2.position.y as f32),
        )
        .color(if self.selected {
            Color::RED
        } else {
            Color::WHITE
        });
    }
}
