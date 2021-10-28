try {
  process.stdout.write = (buffer) => {
    __handleStdout(Buffer.from(buffer).toString());
    return true;
  };

  process.stderr.write = (buffer) => {
    __handleStderr(Buffer.from(buffer).toString());
    return true;
  };

  process.on("uncaughtException", (exception) => {
    console.log(exception);
  })

  const fs = require("fs");
  // globalThis.require = require("module").createRequire(path.join(process.cwd(), "snorestop"));

  globalThis.__deep_require = require;

  const { Snorestop } = require('snorestop-core')
  const snorestop = new Snorestop();
  const selfPackageJson = JSON.parse(fs.readFileSync(path.join(__snorestop_dirname, "package.json"), "utf-8"));
  Object.prototype.log = function (...data) {
    data.unshift(this);
    console.log.apply(console, [...data]);
    return this;
  }

  Object.keys(selfPackageJson.dependencies)
    .map(packageName => require.resolve(packageName + "/package.json"))
    .map(packageJsonPath => JSON.parse(fs.readFileSync(packageJsonPath, "utf-8")))
    .filter(packageJson => packageJson["snorestop-package"])
    .forEach(packageJson => {
      snorestop.load(packageJson, require.resolve(packageJson.name));
    });
} catch (err) {
  __handleStderr(require("util").format(err));
}
