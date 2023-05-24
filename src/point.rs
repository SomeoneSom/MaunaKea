use std::ops;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::Mul<f32> for Point {
    type Output = Point;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Mul<Point> for Point {
    type Output = Point;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl ops::Div<f32> for Point {
    type Output = Point;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn dot(&self, rhs: Point) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    #[inline]
    pub fn distance(&self, rhs: Point) -> f32 {
        f32::sqrt((rhs.x - self.x).powi(2) + (rhs.y - self.y).powi(2))
    }

    #[inline]
    pub fn round(&self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    #[inline]
    pub fn magnitude(&self) -> f32 {
        self.distance(Point::new(0f32, 0f32))
    }

    #[inline]
    pub fn normalize(self) -> Self {
        self / self.magnitude()
    }
}
