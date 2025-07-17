"""
Unit tests for analyzer module.
"""
import os
import tempfile
import shutil
import pytest
from codestate.analyzer import Analyzer
from codestate.visualizer import csv_report, markdown_report, html_report

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

def test_empty_directory():
    temp_dir = tempfile.mkdtemp()
    try:
        analyzer = Analyzer(temp_dir)
        stats = analyzer.analyze()
        assert stats == {}
    finally:
        shutil.rmtree(temp_dir)

def test_non_python_files():
    temp_dir = tempfile.mkdtemp()
    try:
        txt_file = os.path.join(temp_dir, 'readme.txt')
        create_test_file(txt_file, "This is a text file.\nNo code here.")
        analyzer = Analyzer(temp_dir)
        stats = analyzer.analyze()
        # Should not include .txt in stats by default
        assert '.txt' not in stats
    finally:
        shutil.rmtree(temp_dir)

def test_csv_markdown_html_output():
    # Test visualizer output functions with minimal data
    data = [
        {'ext': '.py', 'file_count': 2, 'total_lines': 10, 'comment_lines': 2, 'function_count': 1},
        {'ext': '.js', 'file_count': 1, 'total_lines': 5, 'comment_lines': 1, 'function_count': 1},
    ]
    csv_str = csv_report(data)
    assert 'ext,file_count,total_lines,comment_lines,function_count' in csv_str
    assert '.py' in csv_str
    md_str = markdown_report(data, title='Test')
    assert '| ext | file_count | total_lines | comment_lines | function_count |' in md_str or '|ext|file_count|total_lines|comment_lines|function_count|' in md_str.replace(' ', '')
    html_str = html_report(data, title='Test')
    assert '<table' in html_str and '</table>' in html_str 