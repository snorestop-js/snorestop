"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Snorestop = void 0;
var Snorestop = /** @class */ (function () {
    function Snorestop() {
    }
    Snorestop.prototype.load = function (packageJson, packageIndexPath) {
        require(packageIndexPath);
    };
    return Snorestop;
}());
exports.Snorestop = Snorestop;
