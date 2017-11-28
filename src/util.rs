// From http://gtk-rs.org/tuto/closures
//
// This takes a comma separated list of elements which are then cloned for use
// in a move closure. A `=>` separates the element(s) from the closure.
//
// This allows moved elements to be reused across multiple connected callbacks
// without extra boilerplate code (in most cases).
#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

