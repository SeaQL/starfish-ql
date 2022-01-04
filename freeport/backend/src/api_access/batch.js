class AsyncBatch {
    constructor(
        releaseThreshold,
        releaseCallback = async (items) => {},
        shouldLog = false,
        releaseErrorHandler = console.error
    ) {
        this.items = [];
        this.releaseThreshold = releaseThreshold;
        this.releaseCallback = releaseCallback;
        this.shouldLog = shouldLog;
        this.releaseErrorHandler = releaseErrorHandler;
    }

    clear() {
        this.items = [];
    }

    async push(item) {
        this.items.push(item);
        if (this.items.length >= this.releaseThreshold) {
            await this.release();
        }
    }

    async consumeArray(items, name = "items") {
        const numItems = items.length;
        for (let i = 0; i < numItems; ++i) {
            await this.push(items[i]);
            this.shouldLog
                && (i+1) % 1000 === 0    
                && console.log(`Consuming ${name}: ${i+1}/${numItems}`);
        }
        await this.release();
    }

    async release() {
        try {
            await this.releaseCallback(this.items);
        } catch (e) {
            this.releaseErrorHandler(e);
        }
        this.shouldLog && console.log(`Released batch with ${this.items.length} items.`);
        this.clear();
    }
};

module.exports = {
    AsyncBatch
};