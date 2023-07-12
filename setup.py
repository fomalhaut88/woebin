import sys
import os
import subprocess as sp

from setuptools import find_packages, setup


dll_name = None

if sys.platform == "linux" or sys.platform == "linux2":
    dll_name = 'libwoebin.so'
elif sys.platform == "darwin":
    dll_name = 'woebin.dylib'
elif sys.platform == "win32":
    dll_name = 'woebin.dll'

assert dll_name is not None, f"OS not supported: {sys.platform}"

dll_path = os.path.join('target/release', dll_name)


sp.Popen(["cargo", "build", "--release"]).communicate()


with open('README.md') as f:
    long_description = f.read()


setup(
    name='woebin-python',
    version='0.1.5',
    packages=find_packages(),
    license="MIT",
    description="",
    long_description=long_description,
    long_description_content_type="text/markdown",
    install_requires=[],
    data_files=[('dlls', [dll_path])],
)
