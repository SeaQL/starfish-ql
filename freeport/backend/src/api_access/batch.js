class AsyncBatch {
    constructor(releaseThreshold, releaseCallback = async (items) => {}, shouldLog = false) {
        this.items = [];
        this.releaseThreshold = releaseThreshold;
        this.releaseCallback = releaseCallback;
        this.shouldLog = shouldLog;
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

    async release() {
        this.shouldLog && console.log(`Released batch with ${this.items.length} items.`);
        await this.releaseCallback(this.items);
        this.clear();
    }
};

module.exports = {
    AsyncBatch
};