use std::ops::{Add, BitOr};
// use std::convert::From;
use std::fmt::Debug;
use num_complex::{Complex, ComplexFloat};
use std::f64::consts::PI;



#[derive(Clone)]
enum Connection{
    Impedance(Complex<f64>),
    Series(Box<(Connection, Connection)>),
    Parallel(Box<(Connection, Connection)>),
}

impl Debug for Connection{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Connection::Impedance(v) => {write!(f, "{}", v)}
            Connection::Series(boxed) => {
                let (a, b) = boxed.as_ref();
                write!(f, "({:?} + {:?})", &a, &b)
            }
            Connection::Parallel(boxed) => {
                let (a, b) = boxed.as_ref();
                write!(f, "({:?} | {:?})", &a, &b)
            }
        }
    }
}


// impl From<&Connection> for Connection {
//     fn from(item: &Connection) -> Self {
//         item.clone()
//     }
// }


impl Add for Connection {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        // println!("C + C -> C");
        Connection::new_series(self, other)
    }
}
impl Add for &Connection {
    type Output = Connection;
    fn add(self, other: Self) -> Connection {
        // println!("&C + &C -> C");
        Connection::Series(Box::new((self.clone(), other.clone())))
    }
}
impl Add<&Connection> for Connection {
    type Output = Connection;
    fn add(self, other: &Connection) -> Connection {
        // println!("C + &C -> C");
        Connection::Series(Box::new((self, other.clone())))
    }
}
impl Add<Connection> for &Connection {
    type Output = Connection;
    fn add(self, other: Connection) -> Connection {
        // println!("&C + C -> C");
        Connection::Series(Box::new((self.clone(), other)))
    }
}

impl BitOr for Connection {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Connection::new_parallel(self, other)
    }
}
impl BitOr for &Connection {
    type Output = Connection;
    fn bitor(self, other: Self) -> Connection {
        Connection::Parallel(Box::new((self.clone(), other.clone())))
    }
}
impl BitOr<Connection> for &Connection {
    type Output = Connection;
    fn bitor(self, other: Connection) -> Connection {
        Connection::Parallel(Box::new((self.clone(), other)))
    }
}
impl BitOr<&Connection> for Connection {
    type Output = Connection;
    fn bitor(self, other: &Connection) -> Connection {
        Connection::Parallel(Box::new((self, other.clone())))
    }
}


#[allow(dead_code)]
impl Connection {
    fn new_impedance(re: f64, im: f64) -> Self { Connection::Impedance(Complex::new(re, im)) }
    fn new_series(a: Connection, b: Connection) -> Self { Connection::Series(Box::new((a, b))) }
    fn new_parallel(a: Connection, b: Connection) -> Self { Connection::Parallel(Box::new((a, b))) }

    fn eq(self) -> Self {
        match self {
            Connection::Impedance(_) => {self}
            Connection::Series(boxed) => {
                let (a, b) = *boxed;
                let a = if let Connection::Impedance(_) = a { a } else { a.eq() };
                let b = if let Connection::Impedance(_) = b { b } else { b.eq() };
                match (&a, &b) {
                    (Connection::Impedance(a), Connection::Impedance(b)) => Connection::Impedance(*a + *b),
                    _ => Connection::new_series(a, b)
                }
            }
            Connection::Parallel(boxed) => {
                let (a, b) = *boxed;
                let a = if let Connection::Impedance(_) = a { a } else { a.eq() };
                let b = if let Connection::Impedance(_) = b { b } else { b.eq() };
                match (&a, &b) {
                    (Connection::Impedance(a), Connection::Impedance(b)) => Connection::Impedance(*a * *b /(*a + *b)),
                    _ => Connection::new_parallel(a, b)
                }
            }
            // _ => {self}
        }
    }

    fn i_from_v(&self, v: Complex<f64>) -> Complex<f64> {
        match self {
            Connection::Impedance(z) => {v / z},
            _ => todo!()

        }
    }
     fn v_from_i(&self, i: Complex<f64>) -> Complex<f64> {
        match self {
            Connection::Impedance(z) => {i * z},
            _ => todo!()

        }
    }
    fn unwrap(&self) -> Complex<f64> {
        match self {
            Connection::Impedance(z) => {z.clone()},
            _ => {panic!("Só é possível 'unwrap' Connection::Impedance")}
        }
    }
}


fn main() {

    let ns= 1800.0;
    let nr= 1740.0;
    let s = (ns-nr)/ns;
    let prot = 200.0;
    let v1: Complex<f64> = Complex::new(220.0, 0.0);
    let r1 = Connection::new_impedance(0.41, 0.0);
    let jx1 = Connection::new_impedance(0.0, 0.76);
    let r2 = Connection::new_impedance(0.55, 0.0);
    let jx2 = Connection::new_impedance(0.0, 0.92);
    let rf = Connection::new_impedance(200.0, 0.0);
    let jxm = Connection::new_impedance(0.0, 46.0);
    let rc = Connection::new_impedance(r2.unwrap().re*(1.0-s)/s, 0.0);


    let z = ((&r2 + &rc + &jx2) | (&jxm | &rf)) + (&r1 + &jx1);
    println!("Z_mit: {:?}", &z);
    let mit = z.eq();
    println!("Zeq: {:.3}", &mit.unwrap());
    let i1 = mit.i_from_v(v1);
    println!("I1: {:.3} ∠ {:.3}°", &i1.norm(), &i1.arg());
    let e0 = ((&r2 + &rc + &jx2) | (&jxm | &rf)).eq().v_from_i(i1);
    println!("E0: {:.3} ∠ {:.3}°", &e0.norm(), &e0.arg());
    let i2 = e0 / (&r2 + &rc + &jx2).eq().unwrap();
    println!("I2: {:.3} ∠ {:.3}°", &i2.norm(), &i2.arg());

    let pconv = 3.0 * i2.norm().powi(2) * rc.unwrap().re;
    println!("Pconv: {:.3} W", pconv );
    let peixo = pconv - prot;
    println!("Peixo: {:.3} W = {:.3} hp", peixo, peixo / 745.7 );
    println!("teixo: {:.3} Nm", peixo / (nr * 2.0 * PI / 60.0));
    println!("tind: {:.3} Nm", pconv / (nr * 2.0 * PI / 60.0));
    let pin = 3.0 * (v1 * i1.conj()).re;
    println!("n: {:.3} %", peixo / pin * 100.0);
}

