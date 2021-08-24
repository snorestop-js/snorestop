import ffi from "ffi";

export class Snorestop {
  load(packageJson: any, packageIndexPath: string): void {
    require(packageIndexPath);
  }
}
