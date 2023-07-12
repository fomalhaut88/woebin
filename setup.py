import subprocess as sp

from setuptools import find_packages, setup


sp.Popen(["cargo", "build", "--release"]).communicate()


with open('README.md') as f:
    long_description = f.read()


setup(
    name='woebin-python',
    version='0.1.3',
    packages=find_packages(),
    license="MIT",
    description="",
    long_description=long_description,
    long_description_content_type="text/markdown",
    install_requires=[],
    data_files=[('dlls', ['target/release/woebin.dll'])],
)
