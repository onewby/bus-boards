use std::any::Any;
use crossbeam_channel::unbounded;

struct Pipeline<From, To> {
    tx: crossbeam_channel::Sender<Option<From>>,
    rx: crossbeam_channel::Receiver<Option<To>>,
    nodes: Vec<PipelineNode<dyn Any, dyn Any>>
}

struct PipelineNode<From, To> {
    receiver: crossbeam_channel::Receiver<Option<From>>,
    fct: fn(x: From) -> To,
    sender: crossbeam_channel::Sender<Option<To>>
}

impl <From, To> Pipeline<From, To> {
    fn create(fct: fn(x: From) -> To) -> Pipeline<From, To> {
        let (in_tx, in_rx) = unbounded();
        let (out_tx, out_rx) = unbounded();
        Pipeline {
            tx: in_tx,
            rx: out_rx,
            nodes: vec![
                PipelineNode::<From, To> {
                    receiver: in_rx,
                    fct,
                    sender: out_tx,
                }
            ],
        }
    }

    fn run<Inputs, Outputs>(&self, input: Inputs) -> Outputs
        where Inputs: IntoIterator<Item = From>, Outputs: IntoIterator<Item = From> {
        input.into_iter().for_each(|i| { self.tx.send(i); });
        rayon::scope(|s| {
            self.nodes.iter().for_each(|node| {
                s.spawn(|s| {
                    while let Ok(Some(input)) = node.receiver.recv() {
                        let output = node.fct(input);
                        node.sender.send(Some(output));
                    }
                    node.sender.send(None);
                })
            })
        });
        self.rx.iter().map_while(|o| o)
    }

    fn pipe<NewTo>(self, fct: fn(x: To) -> NewTo) -> Pipeline<From, NewTo> {
        let (tx, rx) = unbounded::<Option<NewTo>>();
        let new_node = PipelineNode::<To, NewTo> {
            receiver: self.rx.clone(),
            fct,
            sender: tx,
        };
        let mut nodes = self.nodes;
        nodes.push(new_node);
        Pipeline {
            tx: self.tx,
            rx,
            nodes
        }
    }
}