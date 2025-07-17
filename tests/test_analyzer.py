"""
Unit tests for analyzer module.
"""
import os
import tempfile
import shutil
import pytest
from codestate.analyzer import Analyzer

def create_test_file(path, content):
    with open(path, 'w', encoding='utf-8') as f:
        f.write(content)

def test_analyzer_basic():
    # Create a temporary directory with test files
    temp_dir = tempfile.mkdtemp()
    try:
        py_file = os.path.join(temp_dir, 'test.py')
        js_file = os.path.join(temp_dir, 'test.js')
        create_test_file(py_file, """
# comment line
def foo():
    pass
if True:
    pass
""")
        create_test_file(js_file, """
// comment line
function bar() {
    // do something
    if (true) {}
}
""")
        analyzer = Analyzer(temp_dir)
        stats = analyzer.analyze()
        # Check Python file stats
        py_stats = stats['.py']
        assert py_stats['file_count'] == 1
        assert py_stats['total_lines'] >= 4
        assert py_stats['comment_lines'] >= 1
        assert py_stats['function_count'] >= 1
        # Check JS file stats
        js_stats = stats['.js']
        assert js_stats['file_count'] == 1
        assert js_stats['total_lines'] >= 5
        assert js_stats['comment_lines'] >= 1
        assert js_stats['function_count'] >= 1
    finally:
        shutil.rmtree(temp_dir) 