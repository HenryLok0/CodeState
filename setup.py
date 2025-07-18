from setuptools import setup, find_packages

setup(
    name='codestate',
    version='0.5.5',
    description='A CLI tool for codebase statistics and ASCII visualization',
    author='Henry Lok',
    packages=find_packages(),
    install_requires=['pathspec', 'openpyxl'],
    entry_points={
        'console_scripts': [
            'codestate=codestate.cli:main',  # This entry point allows users to run `codestate` in the terminal after installation.
        ],
    },
    python_requires='>=3.7',
) 