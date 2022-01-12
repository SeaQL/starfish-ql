import colorsys from "colorsys";

export class Color {

    constructor(hex) {
        this.hex = hex;
        this.rgb = colorsys.hex2Rgb(hex);
        this.hsv = colorsys.hex2Hsv(hex);
    }

    /// 0 <= t <= 1
    interpolateToHex(other, t) {

        const h = interpolateAToB(this.hsv.h, other.hsv.h, t);
        const s = interpolateAToB(this.hsv.s, other.hsv.s, t);
        const v = interpolateAToB(this.hsv.v, other.hsv.v, t);

        return colorsys.hsv2Hex(h, s, v);
    }
}

/// 0 <= t <= 1
const interpolateAToB = (a, b, t) => (a * (1 - t) + b * t);