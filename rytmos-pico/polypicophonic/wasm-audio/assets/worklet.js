class MyProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super();
         
        console.log("hallo wat is hier gaande", options);

        const [arrayBuffer] = options.processorOptions;

        initSync(arrayBuffer);
        console.log("na initsync", wasm);

        audio_main()
    }

    process(inputs, outputs, parameters) {
        process(inputs, outputs, parameters);

        return true;
    }
}

registerProcessor("my-processor", MyProcessor);
