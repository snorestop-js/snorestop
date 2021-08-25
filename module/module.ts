export abstract class SnorestopModule {
  constructor(private readonly pkgJson: any) { }

  getName(): string {
    return this.pkgJson.name;
  }

  getVersion(): string {
    return this.pkgJson.version;
  }
}
