use std::comm::{stream,Port,SharedChan};
// messages to control an Actor
enum ActorMessage{
    Execute,
    Stop,
    SendChan 
}
struct BaseActor{
    // stores SharedChans from other Actors in a vector
    receiver: ~[SharedChan<ActorMessage>],

    // the port for receiving an ActorMessage.
    port: Port<ActorMessage>,
    // a SharedChan to pass it to other Actors.
    chan: SharedChan<ActorMessage>,

    // a port to recv other Chans from other Actors.
    recv_chan: Port<SharedChan<ActorMessage>>,
    // a chan to send the SharedChan to other Actors.
    send_chan: SharedChan<SharedChan<ActorMessage>>
}

//actor trait with default methods
#[allow(default_methods)]
trait Actor{

    // getters that have to implemented on the client 
    fn get_recv_chan<'r>(&'r self) -> &'r Port<SharedChan<ActorMessage>>;
    fn get_port<'r>(&'r self) -> &'r Port<ActorMessage>;
    fn get_chan<'r>(&'r self) -> &'r SharedChan<ActorMessage>;
    fn get_receiver<'r>(&'r mut self) -> &'r mut ~[SharedChan<ActorMessage>];

    // adds a chan in the actor, so that the Actor can communicate with other Actors.
    fn add_receiver(&mut self,a: &SharedChan<ActorMessage>){
        self.get_receiver().push(a.clone());
    }
    //starts the actor, it will listen to incoming messages.
    fn start(&self){
        println("Starting Actor");
        loop{
            if(self.get_port().peek()){
                self.listen_for_messages();
                self.on_receive();
            }  
        }
    }
    // client controlled execute fn.
    fn execute(&self);
    // will be called when a message is received.
    fn on_receive(&self); 
    // match the ActorMessages
    fn listen_for_messages(&self) {
        println("processing message");
        let m = self.get_port().recv();
        match m {
            Execute => self.execute(),
            // doesn't work atm, can't make this fn mutable because
            // I would also need to make .start mutable which makes problems in closures
            // see fn create actor. Any fixes or workarounds?
            SendChan => {
                //let r = self.get_recv_chan().recv();
                //self.add_receiver(&r);
                self.start();
            }
            Stop => fail!("Task was stopped by callee.")
        }
    }
}




impl Actor for BaseActor {
   
    fn get_recv_chan<'r>(&'r self) -> &'r Port<SharedChan<ActorMessage>>{
        &'r(self.recv_chan)
    }  
    fn get_port<'r>(&'r self) -> &'r Port<ActorMessage>{
        &'r(self.port)
    }   
    fn get_receiver<'r>(&'r mut self) -> &'r mut ~[SharedChan<ActorMessage>]{
        &'r mut (self.receiver)
    }
     fn get_chan<'r>(&'r self) -> &'r SharedChan<ActorMessage>{
        &'r(self.chan)
    }
    fn on_receive(&self){
        println("Received an ActorMessage")
    }
    fn execute(&self){
        println("Actor says: \"Hello\"");
    }
}
impl BaseActor {
    // returns the BaseActor itself, a chan to send ActorMessages to the created Actor
    // and a Chan to send SharesChans to Actors.
    fn new() -> (BaseActor,SharedChan<ActorMessage>,SharedChan<SharedChan<ActorMessage>>) {
        println("Creating Actor");
        let (port, chan) = stream::<ActorMessage>();
        let (recv_chan, send_chan) = stream::<SharedChan<ActorMessage>>();
        let chan = SharedChan::new(chan);
        let send_chan = SharedChan::new(send_chan);
        let actor = BaseActor { receiver: ~[],chan: chan.clone(), 
                                port: port,recv_chan: recv_chan, 
                                send_chan: send_chan.clone() };       
        
        (actor,chan,send_chan)     
    }
}
// a fn to create actors and spawn them
// *only for testing, will be replaced by an ActorController*
fn create_actor() -> SharedChan<ActorMessage>{
    let (actor,chan,_) = BaseActor::new();

    do std::task::spawn_unlinked{
       actor.start();
    }
    chan
}

fn main() {
// you control Actors with ActorMessages
let chan = create_actor(); 
chan.send(Execute);
chan.send(Stop);

let chan1 = create_actor(); 
chan1.send(Execute);
chan1.send(Stop);
}
