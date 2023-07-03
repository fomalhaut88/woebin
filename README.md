# woebin

A python library powered by Rust to perform binning procedure based on WOE.

## Installation

```
pip install woebin-python
```

**NOTE**: It works for Windows 11 only. If you want to use it for another platform, please, see below how to build it from source.

## Simple example

```python
from woebin import WoeBinningProc

wbp = WoeBinningProc(5)

# Process categorial binning
wbp.process_categorial(df['series'], df['target'])  # series as integers, target as 0-1 or boolean

# Process numeric binning (values in series are considered as numeric)
#wbp.process_numeric(df['series'], df['target'])

# Final IV
print(wbp.get_iv_total())

# Information about found bins
bins_info = wbp.get_bins_info()
print(bins_info)

# Mapping value->WOE
woe_map = wbp.get_woe_map()
print(woe_map)
```

## Build and install from source

1. Make sure Rust nightly is installed.
2. Clone the repository: `git clone --depth=1 https://github.com/fomalhaut88/woebin.git`
3. Go to `woebin` directory: `cd woebin`
4. Build the project: `python setup.py sdist`

After that the package can be installed with the command:

```
pip install path/to/woebin-python-<VERSION>.tar.gz
```
