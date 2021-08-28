#[macro_export]
macro_rules! set {
    ($object: expr, $cx: expr, $name: expr, $value: expr) => {
        {
            let name = $name;
            let value = $value;
            $object.set($cx, name, value).expect(format!("failed to set value on {}", stringify!($object)).as_str())
        }
    };
}

#[macro_export]
macro_rules! get {
    ($object: expr, $cx: expr, $name: expr) => {
        {
            let name = $name;
            $object.get($cx, name).expect(format!("failed to get value on {}", stringify!($object)).as_str())
        }
    };
}