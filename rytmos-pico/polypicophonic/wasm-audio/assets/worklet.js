
class MyProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super();

        const [arrayBuffer] = options.processorOptions;
        initSync(arrayBuffer);
        init_logging();

        // this.port.addEventListener('message', (...args) => console.log(...args))
        // this.port.addEventListener('messageerror', (...args) => console.log(...args))
        this.port.start();

        this.processor = new Processor(this.port);

    }

    process(inputs, outputs, parameters) {
        this.processor.process(inputs, outputs, parameters);

        return true;
    }
}

registerProcessor("my-processor", MyProcessor);
