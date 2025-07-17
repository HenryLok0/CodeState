import os
import pathlib
import re
from collections import defaultdict
import concurrent.futures
import hashlib
import subprocess

class Analyzer:
    """
    Analyzer for code statistics: lines of code, comment density, function complexity.
    """
    def __init__(self, root_dir, file_types=None, exclude_dirs=None):
        # root_dir: the directory to analyze
        # file_types: list of file extensions to include (e.g., ['.py', '.js'])
        # exclude_dirs: list of directory names to exclude
        self.root_dir = pathlib.Path(root_dir)
        self.file_types = file_types or [
            '.py', '.js', '.ts', '.java', '.cpp', '.c', '.go', '.rb', '.php', '.cs', '.sh'
        ]
        self.exclude_dirs = set(exclude_dirs or ['.git', 'venv', 'node_modules'])
        self.stats = defaultdict(lambda: {
            'file_count': 0,
            'total_lines': 0,
            'comment_lines': 0,
            'function_count': 0,
            'complexity': 0,
            'todo_count': 0,
            'blank_lines': 0,
            'comment_only_lines': 0,
            'code_lines': 0
        })
        self.file_details = []  # List of per-file stats
        self.duplicates = []  # List of duplicate code info

    def analyze(self):
        # Recursively scan files and collect statistics (multithreaded)
        files = [file_path for file_path in self._iter_files(self.root_dir) if file_path.suffix in self.file_types]
        with concurrent.futures.ThreadPoolExecutor() as executor:
            list(executor.map(self._analyze_file, files))
        # Calculate comment density and average complexity
        for ext, data in self.stats.items():
            if data['file_count'] > 0:
                data['comment_density'] = data['comment_lines'] / data['total_lines'] if data['total_lines'] else 0
                data['avg_complexity'] = data['complexity'] / data['function_count'] if data['function_count'] else 0
                data['function_avg_length'] = (data['total_lines'] / data['function_count']) if data['function_count'] else 0
        self._detect_duplicates()
        self._detect_git_authors()
        # Find max/min file by total_lines
        if self.file_details:
            self.max_file = max(self.file_details, key=lambda x: x['total_lines'])
            self.min_file = min(self.file_details, key=lambda x: x['total_lines'])
        else:
            self.max_file = self.min_file = None
        return self.stats

    def _detect_duplicates(self, block_size=5):
        # Detect duplicate code blocks of block_size lines across all files
        block_map = {}  # hash -> list of (file, start_line, block)
        for file_stat in self.file_details:
            path = file_stat['path']
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    lines = f.readlines()
                    for i in range(len(lines) - block_size + 1):
                        block = ''.join(lines[i:i+block_size])
                        block_hash = hashlib.md5(block.encode('utf-8')).hexdigest()
                        block_map.setdefault(block_hash, []).append((path, i+1, block))
            except Exception:
                continue
        # Collect duplicates (appearing in 2+ places)
        self.duplicates = [v for v in block_map.values() if len(v) > 1]

    def get_duplicates(self):
        # Return list of duplicate code blocks
        return self.duplicates

    def _detect_git_authors(self):
        # If .git exists, get main author and last modifier for each file
        git_dir = self.root_dir / '.git'
        if not git_dir.exists():
            self.git_authors = None
            return
        self.git_authors = {}
        for file_stat in self.file_details:
            path = file_stat['path']
            rel_path = os.path.relpath(path, self.root_dir)
            try:
                # Get main author (most commits)
                cmd = ['git', '-C', str(self.root_dir), 'log', '--format=%an', rel_path]
                authors = subprocess.check_output(cmd, encoding='utf-8', errors='ignore').splitlines()
                if authors:
                    main_author = max(set(authors), key=authors.count)
                    last_author = authors[0]
                else:
                    main_author = last_author = None
            except Exception:
                main_author = last_author = None
            self.git_authors[path] = {'main_author': main_author, 'last_author': last_author}

    def get_git_authors(self):
        # Return dict: file path -> {'main_author', 'last_author'}
        return getattr(self, 'git_authors', None)

    def _iter_files(self, root):
        # Generator that yields files, skipping excluded directories
        for dirpath, dirnames, filenames in os.walk(root):
            # Remove excluded directories in-place
            dirnames[:] = [d for d in dirnames if d not in self.exclude_dirs]
            for filename in filenames:
                yield pathlib.Path(dirpath) / filename

    def _analyze_file(self, file_path):
        # Analyze a single file for statistics
        ext = file_path.suffix
        total_lines = 0
        comment_lines = 0
        function_count = 0
        complexity = 0
        todo_count = 0
        blank_lines = 0
        comment_only_lines = 0
        code_lines = 0
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                for line in f:
                    total_lines += 1
                    line_strip = line.strip()
                    if not line_strip:
                        blank_lines += 1
                        continue
                    # Simple comment detection (supports //, #, /*, *)
                    is_comment = line_strip.startswith('#') or line_strip.startswith('//') or line_strip.startswith('/*') or line_strip.startswith('*')
                    if is_comment:
                        comment_lines += 1
                        if len(line_strip) == len(line):
                            comment_only_lines += 1
                    # TODO/FIXME detection
                    if 'TODO' in line_strip or 'FIXME' in line_strip:
                        todo_count += 1
                    # Simple function detection (Python, JS, Java, C-like)
                    if re.match(r'^(def |function |void |int |float |double |public |private |static |func )', line_strip):
                        function_count += 1
                        complexity += self._estimate_complexity(line_strip)
                    # Count code lines (not blank, not only comment)
                    if not is_comment:
                        code_lines += 1
        except Exception as e:
            pass  # Ignore unreadable files
        self.stats[ext]['file_count'] += 1
        self.stats[ext]['total_lines'] += total_lines
        self.stats[ext]['comment_lines'] += comment_lines
        self.stats[ext]['function_count'] += function_count
        self.stats[ext]['complexity'] += complexity
        self.stats[ext]['todo_count'] += todo_count
        self.stats[ext]['blank_lines'] += blank_lines
        self.stats[ext]['comment_only_lines'] += comment_only_lines
        self.stats[ext]['code_lines'] += code_lines
        function_avg_length = (total_lines / function_count) if function_count else 0
        file_stat = {
            'path': str(file_path),
            'ext': ext,
            'total_lines': total_lines,
            'comment_lines': comment_lines,
            'function_count': function_count,
            'complexity': complexity,
            'function_avg_length': function_avg_length,
            'todo_count': todo_count,
            'blank_lines': blank_lines,
            'comment_only_lines': comment_only_lines,
            'code_lines': code_lines
        }
        self.file_details.append(file_stat)

    def _estimate_complexity(self, line):
        # Simple cyclomatic complexity estimation: count keywords
        keywords = ['if ', 'for ', 'while ', 'case ', '&&', '||', 'elif ', 'except ', 'catch ']
        return 1 + sum(line.count(k) for k in keywords)

    def get_file_details(self):
        # Return per-file statistics
        return self.file_details

    def get_max_min_stats(self):
        # Return file with most/least lines
        return {'max_file': self.max_file, 'min_file': self.min_file} 