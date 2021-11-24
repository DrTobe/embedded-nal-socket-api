fn main() {
    {
        // &RefCell impls mutex_trait::Mutex
        println!("Testing DriverA with &RefCell");
        let driver = core::cell::RefCell::new(DriverA::new());
        let mut stack = TcpClientStack::new(&driver);
        // TODO I do not like that the user could do nasty things like:
        // let borrow = driver.borrow_mut();
        // stack.socket(); // panic: 'already borrowed: BorrowMutError'
        let mut socket0 = stack.socket().unwrap();
        let mut socket1 = stack.socket().unwrap();
        let _ = socket0.connect(SocketAddr);
        let _ = socket1.connect(SocketAddr);
        drop(socket0);
        let _socket0_again = stack.socket();
    }

    // Using another Mutex impl
    println!("\nTesting DriverB with StdMutex (i.e. std::sync::Mutex)");
    let driver = StdMutex(std::sync::Mutex::new(DriverB::new()));
    let mut stack = TcpClientStack::new(&driver);
    let _ = stack.socket().unwrap().connect(SocketAddr);
}

//
// NalDriver trait definition
//

pub trait NalDriver {
    type SocketIdentifier: Copy;
    fn socket(&mut self) -> Result<Self::SocketIdentifier, Error>;
    fn connect(&mut self, socket: Self::SocketIdentifier, remote: SocketAddr) -> Result<(), Error>;
    // + send, receive
    fn close(&mut self, socket: Self::SocketIdentifier);
}

//
// This is what could be supplied by embedded-nal
//

#[derive(Clone, Copy)]
pub struct TcpClientStack<D, M>
where
    D: NalDriver,
    M: mutex_trait::Mutex<Data = D> + Clone,
{
    driver: M,
}

impl<D, M> TcpClientStack<D, M>
where
    D: NalDriver,
    M: mutex_trait::Mutex<Data = D> + Clone,
{
    pub fn new(driver: M) -> Self {
        TcpClientStack { driver }
    }

    fn socket(&mut self) -> Result<TcpSocketHandle<D, M>, Error> {
        Ok(TcpSocketHandle {
            socket: self.driver.lock(|d| d.socket())?,
            driver: self.driver.clone(),
        })
    }
}

/// Socket handle type for a TCP socket
pub struct TcpSocketHandle<D, M>
where
    D: NalDriver,
    M: mutex_trait::Mutex<Data = D> + Clone,
{
    socket: D::SocketIdentifier,
    driver: M,
}

impl<D, M> TcpSocketHandle<D, M>
where
    D: NalDriver,
    M: mutex_trait::Mutex<Data = D> + Clone,
{
    fn connect(&mut self, remote: SocketAddr) -> Result<(), Error> {
        self.driver.lock(|d| d.connect(self.socket, remote))
    }

    // + send, receive
}

impl<D, M> Drop for TcpSocketHandle<D, M>
where
    D: NalDriver,
    M: mutex_trait::Mutex<Data = D> + Clone,
{
    fn drop(&mut self) {
        self.driver.lock(|d| d.close(self.socket))
    }
}

//
// This is what the driver developer A has to do
//

/// A Network Driver that supports multiple sockets
pub struct DriverA {
    sockets: [SocketA; 12],
}

impl DriverA {
    pub fn new() -> Self {
        DriverA {
            sockets: [SocketA {
                state: SocketAState::Available,
            }; 12],
        }
    }
}

impl NalDriver for DriverA {
    type SocketIdentifier = usize;

    fn socket(&mut self) -> Result<usize, Error> {
        for i in 0..12 {
            if let SocketAState::Available = self.sockets[i].state {
                println!("Using socket {}", i);
                self.sockets[i].state = SocketAState::Used;
                return Ok(i);
            }
        }
        Err(Error)
    }

    fn connect(&mut self, socket_index: usize, _remote: SocketAddr) -> Result<(), Error> {
        // ...
        println!("Connecting socket {}", socket_index);
        Ok(())
    }

    fn close(&mut self, socket_index: usize) {
        // ...
        println!("Closing socket {}", socket_index);
        self.sockets[socket_index].state = SocketAState::Available;
    }

    // + send, receive
}

// This is just Clone+Copy to simplify array initialization in DriverA::new
#[derive(Clone, Copy)]
struct SocketA {
    state: SocketAState,
}

#[derive(Clone, Copy, PartialEq)]
enum SocketAState {
    Available,
    Used,
    _Open, // close -> Available again
}

//
// This is what the driver developer B has to do
//

/// A Network Driver that supports a single socket
pub struct DriverB {
    socket_available: bool,
}

impl DriverB {
    pub fn new() -> Self {
        DriverB {
            socket_available: true,
        }
    }
}

impl NalDriver for DriverB {
    type SocketIdentifier = ();

    fn socket(&mut self) -> Result<(), Error> {
        if self.socket_available {
            println!("Using single socket");
            self.socket_available = false;
            Ok(())
        } else {
            Err(Error)
        }
    }

    fn connect(&mut self, _socket: (), _remote: SocketAddr) -> Result<(), Error> {
        // ...
        println!("Connecting single socket");
        Ok(())
    }

    fn close(&mut self, _socket: ()) {
        // ...
        println!("Closing single socket");
        self.socket_available = true;
    }

    // + send, receive
}

//
// Type definitions required so that the above compiles successfully
//

pub struct SocketAddr;
#[derive(Debug)]
pub struct Error;

pub struct StdMutex<D>(std::sync::Mutex<D>);

impl<D> mutex_trait::Mutex for &'_ StdMutex<D> {
    type Data = D;

    fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        let mut d = self.0.lock().unwrap();
        f(&mut d)
    }
}
