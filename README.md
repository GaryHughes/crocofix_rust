# CrocoFIX



# Orchestrations

The lexicographer relies on [fixorchestra](https://github.com/GaryHughes/fixorchestra) which has been git subtree merged as follows.
```sh
git remote add -f fixorchestra https://github.com/GaryHughes/fixorchestra.git
git subtree add --prefix fixorchestra fixorchestra master --squash
# To push changes upstream
git subtree push --prefix fixorchestra fixorchestra master --squash
# To pull changes from upstream
git subtree pull --prefix fixorchestra fixorchestra master --squash
```

## lexicographer

The lexicographer is a set of [Python](https://python.org) scripts that parses the orchestration XML and generates a set of modules to allow for easy consumption of the orchestration metdata in Rust programs. Details can be found [here](https://github.com/GaryHughes/crocofix_rust/blob/master/lexicographer/README.md). The generated types rely on common code in the dictionary crate.
