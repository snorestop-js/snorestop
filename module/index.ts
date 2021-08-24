export { SnorestopModule } from "./module";

export class Snorestop {
  load(packageJson: any, packageIndexPath: string): void {
    new (require(packageIndexPath).default)();
  }
}
