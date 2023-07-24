import sys
import os
import json
import subprocess as sp

from setuptools import setup


PACKAGE_NAME = 'woebin'


def build_src():
    sp.Popen(["cargo", "build", "--release"]).communicate()


def get_version():
    try:
        out, _ = sp.Popen(["cargo", "metadata"], stdout=sp.PIPE).communicate()

    except FileNotFoundError:
        return __import__(f"{PACKAGE_NAME}").__version__

    else:
        metadata = json.loads(out.decode())

        for package in metadata['packages']:
            if package['name'] == PACKAGE_NAME:
                version = package['version']

        path_version_py = os.path.join(PACKAGE_NAME, 'version.py')
        with open(path_version_py, 'w') as f:
            print(f"__version__ = \"{version}\"", file=f)

        return version


def get_long_description():
    with open('README.md') as f:
        return f.read()


def get_dll_paths():
    dll_name = None

    if sys.platform == "linux" or sys.platform == "linux2":
        dll_name = f'lib{PACKAGE_NAME}.so'
    elif sys.platform == "darwin":
        dll_name = f'lib{PACKAGE_NAME}.dylib'
    elif sys.platform == "win32":
        dll_name = f'{PACKAGE_NAME}.dll'

    assert dll_name is not None, f"OS not supported: {sys.platform}"

    dll_path = os.path.join('target/release', dll_name)

    return [dll_path]


# Build from source
build_src()


# Setup
setup(
    name='woebin-python',
    version=get_version(),
    packages=[PACKAGE_NAME],
    license="MIT",
    description="",
    long_description=get_long_description(),
    long_description_content_type="text/markdown",
    install_requires=[],
    data_files=[('dlls', get_dll_paths())],
)
