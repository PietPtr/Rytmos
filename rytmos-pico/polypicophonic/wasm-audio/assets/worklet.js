

class MyProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super();
        console.log("hallo wat is hier gaande", options);

        __wbg_init();

        console.log("na initsync");
    }

    process(inputs, outputs, parameters) {
        // return this.processor.process(outputs[0][0]);
    }
}

registerProcessor("my-processor", MyProcessor);
