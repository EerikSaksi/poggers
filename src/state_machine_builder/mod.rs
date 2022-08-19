trait State {
    fn process();
}

struct JsonBuilder<S: State> {
    state: S,
}

struct ParentState {}
impl State for ParentState {
    fn process() {
        
    }
}


