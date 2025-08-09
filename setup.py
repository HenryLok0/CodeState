from setuptools import setup, find_packages
from pathlib import Path

readme_path = Path(__file__).parent / "README.md"
long_description = readme_path.read_text(encoding="utf-8") if readme_path.exists() else ""

setup(
    name='codestate',
    version='0.9.4',
    description='A CLI tool for codebase statistics and ASCII visualization',
    long_description=long_description,
    long_description_content_type='text/markdown',
    author='Henry Lok',
    url='https://github.com/HenryLok0/CodeState',
    project_urls={
        'Source': 'https://github.com/HenryLok0/CodeState',
        'Tracker': 'https://github.com/HenryLok0/CodeState/issues',
        'Documentation': 'https://github.com/HenryLok0/CodeState#readme',
    },
    packages=find_packages(),
    install_requires=['pathspec', 'openpyxl'],
    entry_points={
        'console_scripts': [
            'codestate=codestate.cli:main',
        ],
    },
    keywords=['code metrics', 'cli', 'ascii', 'static analysis', 'loc', 'complexity'],
    classifiers=[
        'Programming Language :: Python :: 3',
        'License :: OSI Approved :: MIT License',
        'Operating System :: OS Independent',
        'Topic :: Software Development :: Quality Assurance',
        'Topic :: Software Development :: Libraries :: Python Modules',
        'Environment :: Console',
        'Intended Audience :: Developers',
    ],
    python_requires='>=3.7',
    include_package_data=True,
)