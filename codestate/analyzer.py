import os
import pathlib
import re
from collections import defaultdict
import concurrent.futures
import hashlib
import subprocess
import ast

class Analyzer:
    """
    Analyzer for code statistics: lines of code, comment density, function complexity.
    """
    def __init__(self, root_dir, file_types=None, exclude_dirs=None):
        # root_dir: the directory to analyze
        # file_types: list of file extensions to include (e.g., ['.py', '.js'])
        # exclude_dirs: list of directory names to exclude
        self.root_dir = pathlib.Path(root_dir)
        self.file_types = file_types  # None means auto-detect all extensions
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

    def analyze(self, regex_rules=None):
        # Recursively scan files and collect statistics (multithreaded, thread-safe aggregation)
        if self.file_types is None:
            files = [file_path for file_path in self._iter_files(self.root_dir) if file_path.suffix]
        else:
            files = [file_path for file_path in self._iter_files(self.root_dir) if file_path.suffix in self.file_types]
        def analyze_file_safe(file_path):
            try:
                return self._analyze_file_threadsafe(file_path)
            except Exception as e:
                print(f"Error analyzing {file_path}: {e}")
                return None
        results = []
        with concurrent.futures.ThreadPoolExecutor() as executor:
            for res in executor.map(analyze_file_safe, files):
                if res:
                    results.append(res)
        # Aggregate results
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
        self.file_details = []
        for stat, file_stat in results:
            ext = file_stat['ext']
            for k in stat:
                self.stats[ext][k] += stat[k]
            self.file_details.append(file_stat)
        # Calculate comment density and average complexity
        for ext, data in self.stats.items():
            if data['file_count'] > 0:
                data['comment_density'] = data['comment_lines'] / data['total_lines'] if data['total_lines'] else 0
                data['avg_complexity'] = data['complexity'] / data['function_count'] if data['function_count'] else 0
                data['function_avg_length'] = (data['total_lines'] / data['function_count']) if data['function_count'] else 0
        self._detect_duplicates()
        self._detect_git_authors()
        self._check_naming_conventions()
        self._extract_api_doc_summaries()
        self._detect_large_warnings()
        self._analyze_git_hotspots()
        self._analyze_file_trends()
        self._analyze_refactor_suggestions()
        self._analyze_openapi()
        self._analyze_style_issues()
        self._analyze_contributor_stats()
        self._analyze_advanced_security_issues()
        if regex_rules:
            self._check_regex_rules(regex_rules)
        self._generate_health_report()
        self._generate_grouped_stats()
        self._detect_unused_defs()
        self._analyze_api_param_type_stats()
        self._scan_security_issues()
        # Find max/min file by total_lines
        if self.file_details:
            self.max_file = max(self.file_details, key=lambda x: x['total_lines'])
            self.min_file = min(self.file_details, key=lambda x: x['total_lines'])
        else:
            self.max_file = self.min_file = None
        return self.stats

    def _analyze_file_threadsafe(self, file_path):
        # Returns (stat_dict, file_stat) for aggregation
        ext = file_path.suffix
        total_lines = 0
        comment_lines = 0
        function_count = 0
        complexity = 0
        todo_count = 0
        blank_lines = 0
        comment_only_lines = 0
        code_lines = 0
        size = 0
        try:
            size = os.path.getsize(file_path)
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                for line in f:
                    total_lines += 1
                    line_strip = line.strip()
                    if not line_strip:
                        blank_lines += 1
                        continue
                    is_comment = line_strip.startswith('#') or line_strip.startswith('//') or line_strip.startswith('/*') or line_strip.startswith('*')
                    if is_comment:
                        comment_lines += 1
                        if len(line_strip) == len(line):
                            comment_only_lines += 1
                    if 'TODO' in line_strip or 'FIXME' in line_strip:
                        todo_count += 1
                    if re.match(r'^(def |function |void |int |float |double |public |private |static |func )', line_strip):
                        function_count += 1
                        complexity += self._estimate_complexity(line_strip)
                    if not is_comment:
                        code_lines += 1
        except Exception as e:
            print(f"File read error in {file_path}: {e}")
        stat = {
            'file_count': 1,
            'total_lines': total_lines,
            'comment_lines': comment_lines,
            'function_count': function_count,
            'complexity': complexity,
            'todo_count': todo_count,
            'blank_lines': blank_lines,
            'comment_only_lines': comment_only_lines,
            'code_lines': code_lines
        }
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
            'code_lines': code_lines,
            'size': size
        }
        return stat, file_stat

    def _scan_security_issues(self):
        # Scan for common insecure patterns
        import re
        patterns = [
            (r'\beval\s*\(', 'Use of eval()'),
            (r'\bexec\s*\(', 'Use of exec()'),
            (r'\bpickle\.load\s*\(', 'Use of pickle.load()'),
            (r'\bos\.system\s*\(', 'Use of os.system()'),
            (r'\bsubprocess\.Popen\s*\(', 'Use of subprocess.Popen()'),
            (r'\binput\s*\(', 'Use of input()'),
            (r'password\s*=\s*["\"][^"\"]+["\"]', 'Hardcoded password'),
            (r'token\s*=\s*["\"][^"\"]+["\"]', 'Hardcoded token'),
            (r'secret\s*=\s*["\"][^"\"]+["\"]', 'Hardcoded secret'),
            (r'api[_-]?key\s*=\s*["\"][^"\"]+["\"]', 'Hardcoded API key'),
        ]
        issues = []
        for file_stat in self.file_details:
            path = file_stat['path']
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    for lineno, line in enumerate(f, 1):
                        for pat, desc in patterns:
                            if re.search(pat, line, re.IGNORECASE):
                                issues.append({'file': path, 'line': lineno, 'desc': desc, 'content': line.strip(), 'note': 'Potentially false positive: regex match may include comments or strings.'})
            except Exception:
                continue
        self.security_issues = issues

    def get_security_issues(self):
        # Return list of detected security issues
        return getattr(self, 'security_issues', [])

    def _generate_grouped_stats(self):
        # Group stats by top-level directory and by extension
        from collections import defaultdict
        self.grouped_by_dir = defaultdict(lambda: {'file_count': 0, 'total_lines': 0, 'comment_lines': 0, 'function_count': 0})
        self.grouped_by_ext = defaultdict(lambda: {'file_count': 0, 'total_lines': 0, 'comment_lines': 0, 'function_count': 0})
        for f in self.file_details:
            # By directory (top-level folder)
            rel_path = os.path.relpath(f['path'], self.root_dir)
            parts = rel_path.split(os.sep)
            top_dir = parts[0] if len(parts) > 1 else '.'
            self.grouped_by_dir[top_dir]['file_count'] += 1
            self.grouped_by_dir[top_dir]['total_lines'] += f['total_lines']
            self.grouped_by_dir[top_dir]['comment_lines'] += f['comment_lines']
            self.grouped_by_dir[top_dir]['function_count'] += f['function_count']
            # By extension
            ext = f['ext']
            self.grouped_by_ext[ext]['file_count'] += 1
            self.grouped_by_ext[ext]['total_lines'] += f['total_lines']
            self.grouped_by_ext[ext]['comment_lines'] += f['comment_lines']
            self.grouped_by_ext[ext]['function_count'] += f['function_count']

    def get_grouped_stats(self, by='dir'):
        # Return grouped stats by 'dir' or 'ext'
        if by == 'dir':
            return dict(self.grouped_by_dir)
        elif by == 'ext':
            return dict(self.grouped_by_ext)
        else:
            return {}

    def _generate_health_report(self):
        # Compute a health score and suggestions
        score = 100
        suggestions = []
        # Comment density
        avg_comment_density = 0
        total_lines = 0
        total_comments = 0
        for ext, data in self.stats.items():
            total_lines += data['total_lines']
            total_comments += data['comment_lines']
        if total_lines:
            avg_comment_density = total_comments / total_lines
        if avg_comment_density < 0.05:
            score -= 10
            suggestions.append('Increase comment density (currently low).')
        # Duplicate code
        if self.duplicates and len(self.duplicates) > 0:
            score -= 10
            suggestions.append('Reduce duplicate code blocks.')
        # Large files/functions
        large_warn = getattr(self, 'large_warnings', {'files': [], 'functions': []})
        if large_warn['files']:
            score -= 5
            suggestions.append('Refactor or split large files.')
        if large_warn['functions']:
            score -= 5
            suggestions.append('Refactor or split large functions.')
        # TODO/FIXME
        todo_count = sum(f['todo_count'] for f in self.file_details)
        if todo_count > 10:
            score -= 5
            suggestions.append('Resolve outstanding TODO/FIXME comments.')
        # Naming violations
        naming_violations = getattr(self, 'naming_violations', [])
        if naming_violations:
            score -= 5
            suggestions.append('Fix function/class naming convention violations.')
        # Complexity
        avg_complexity = 0
        total_func = 0
        total_cplx = 0
        for ext, data in self.stats.items():
            total_func += data['function_count']
            total_cplx += data['complexity']
        if total_func:
            avg_complexity = total_cplx / total_func
        if avg_complexity > 3:
            score -= 5
            suggestions.append('Reduce average function complexity.')
        # Bound score
        score = max(0, min(100, score))
        self.health_report = {
            'score': score,
            'avg_comment_density': avg_comment_density,
            'avg_complexity': avg_complexity,
            'todo_count': todo_count,
            'naming_violations': len(naming_violations),
            'duplicate_blocks': len(self.duplicates) if self.duplicates else 0,
            'large_files': len(large_warn['files']),
            'large_functions': len(large_warn['functions']),
            'suggestions': suggestions
        }

    def get_health_report(self):
        # Return health report dict
        return getattr(self, 'health_report', None)

    def _analyze_git_hotspots(self):
        # Analyze git log to find most frequently changed files
        git_dir = self.root_dir / '.git'
        if not git_dir.exists():
            self.git_hotspots = None
            return
        import subprocess
        from collections import Counter
        try:
            cmd = ['git', '-C', str(self.root_dir), 'log', '--name-only', '--pretty=format:']
            output = subprocess.check_output(cmd, encoding='utf-8', errors='ignore', stderr=subprocess.DEVNULL)
            files = [line.strip() for line in output.splitlines() if line.strip()]
            counter = Counter(files)
            self.git_hotspots = counter.most_common()
        except Exception:
            self.git_hotspots = None  # Suppress all git errors

    def get_git_hotspots(self, top_n=10):
        # Return list of (file, commit_count) for the most frequently changed files
        if not getattr(self, 'git_hotspots', None):
            return []
        return self.git_hotspots[:top_n]

    def _detect_large_warnings(self, threshold_file=300, threshold_func=50):
        # Warn for large files and large functions (Python only for functions)
        self.large_warnings = {'files': [], 'functions': []}
        for file_stat in self.file_details:
            if file_stat['total_lines'] > threshold_file:
                self.large_warnings['files'].append({
                    'file': file_stat['path'],
                    'lines': file_stat['total_lines'],
                    'threshold': threshold_file
                })
            if file_stat['ext'] == '.py':
                path = file_stat['path']
                try:
                    with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                        tree = ast.parse(f.read(), filename=path)
                    for node in ast.walk(tree):
                        if isinstance(node, ast.FunctionDef):
                            start = node.lineno
                            end = max([n.lineno for n in ast.walk(node) if hasattr(n, 'lineno')], default=start)
                            func_lines = end - start + 1
                            if func_lines > threshold_func:
                                self.large_warnings['functions'].append({
                                    'file': path,
                                    'function': node.name,
                                    'lines': func_lines,
                                    'line': start,
                                    'threshold': threshold_func
                                })
                except Exception as e:
                    print(f"AST parse error in {path}: {e}")
                    continue

    def get_large_warnings(self, threshold_file=300, threshold_func=50):
        # Return large file/function warnings
        return getattr(self, 'large_warnings', {'files': [], 'functions': []})

    def _extract_api_doc_summaries(self):
        # Only extract for Python files
        self.api_doc_summaries = []
        for file_stat in self.file_details:
            if file_stat['ext'] != '.py':
                continue
            path = file_stat['path']
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    tree = ast.parse(f.read(), filename=path)
                for node in ast.walk(tree):
                    if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
                        doc = ast.get_docstring(node)
                        self.api_doc_summaries.append({
                            'type': 'function' if isinstance(node, ast.FunctionDef) else 'class',
                            'name': node.name,
                            'file': path,
                            'line': node.lineno,
                            'docstring': doc or ''
                        })
            except Exception as e:
                print(f"AST parse error in {path}: {e}")
                continue

    def get_api_doc_summaries(self):
        # Return list of API doc summaries
        return getattr(self, 'api_doc_summaries', [])

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
                authors = subprocess.check_output(cmd, encoding='utf-8', errors='ignore', stderr=subprocess.DEVNULL).splitlines()
                if authors:
                    main_author = max(set(authors), key=authors.count)
                    last_author = authors[0]
                else:
                    main_author = last_author = None
            except Exception:
                main_author = last_author = None  # Suppress all git errors
            self.git_authors[path] = {'main_author': main_author, 'last_author': last_author}

    def get_git_authors(self):
        # Return dict: file path -> {'main_author', 'last_author'}
        return getattr(self, 'git_authors', None)

    def _check_naming_conventions(self):
        # Only check Python files for now
        self.naming_violations = []
        for file_stat in self.file_details:
            if file_stat['ext'] != '.py':
                continue
            path = file_stat['path']
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    tree = ast.parse(f.read(), filename=path)
                for node in ast.walk(tree):
                    if isinstance(node, ast.FunctionDef):
                        if not self._is_snake_case(node.name):
                            self.naming_violations.append({'type': 'function', 'name': node.name, 'file': path, 'line': node.lineno, 'rule': 'snake_case'})
                    if isinstance(node, ast.ClassDef):
                        if not self._is_pascal_case(node.name):
                            self.naming_violations.append({'type': 'class', 'name': node.name, 'file': path, 'line': node.lineno, 'rule': 'PascalCase'})
            except Exception as e:
                print(f"AST parse error in {path}: {e}")
                continue

    def _is_snake_case(self, name):
        # Check if name is snake_case
        import re
        return bool(re.match(r'^[a-z_][a-z0-9_]*$', name))

    def _is_pascal_case(self, name):
        # Check if name is PascalCase
        import re
        return bool(re.match(r'^[A-Z][a-zA-Z0-9]*$', name))

    def get_naming_violations(self):
        # Return list of naming violations
        return getattr(self, 'naming_violations', [])

    def _check_regex_rules(self, regex_rules):
        # regex_rules: list of regex strings
        import re
        self.regex_matches = []
        for file_stat in self.file_details:
            path = file_stat['path']
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    for lineno, line in enumerate(f, 1):
                        for rule in regex_rules:
                            if re.search(rule, line):
                                self.regex_matches.append({
                                    'file': path,
                                    'line': lineno,
                                    'rule': rule,
                                    'content': line.strip()
                                })
            except Exception:
                continue

    def get_regex_matches(self):
        # Return list of regex matches
        return getattr(self, 'regex_matches', [])

    def _detect_unused_defs(self):
        # Only for Python files: find functions/classes defined but never used
        import ast
        defined = set()
        used = set()
        for file_stat in self.file_details:
            if file_stat['ext'] != '.py':
                continue
            path = file_stat['path']
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    tree = ast.parse(f.read(), filename=path)
                for node in ast.walk(tree):
                    if isinstance(node, ast.FunctionDef) or isinstance(node, ast.ClassDef):
                        defined.add((node.name, path, node.lineno, type(node).__name__))
                    # Find all function/class usage (calls, instantiations)
                    if isinstance(node, ast.Call):
                        if hasattr(node.func, 'id'):
                            used.add(node.func.id)
                        elif hasattr(node.func, 'attr'):
                            used.add(node.func.attr)
                    if isinstance(node, ast.Attribute):
                        used.add(node.attr)
            except Exception as e:
                print(f"AST parse error in {path}: {e}")
                continue
        self.unused_defs = [
            {'name': name, 'file': file, 'line': line, 'type': typ}
            for (name, file, line, typ) in defined if name not in used
        ]

    def get_unused_defs(self):
        # Return list of unused function/class definitions
        return getattr(self, 'unused_defs', [])

    def _analyze_api_param_type_stats(self):
        # For Python files: count function parameters and type annotation coverage
        import ast
        total_funcs = 0
        total_params = 0
        total_annotated_params = 0
        total_annotated_returns = 0
        for file_stat in self.file_details:
            if file_stat['ext'] != '.py':
                continue
            path = file_stat['path']
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    tree = ast.parse(f.read(), filename=path)
                for node in ast.walk(tree):
                    if isinstance(node, ast.FunctionDef):
                        total_funcs += 1
                        # Support posonlyargs, args, kwonlyargs
                        all_args = []
                        if hasattr(node.args, 'posonlyargs'):
                            all_args.extend(node.args.posonlyargs)
                        all_args.extend(node.args.args)
                        all_args.extend(node.args.kwonlyargs)
                        for arg in all_args:
                            total_params += 1
                            if arg.annotation is not None:
                                total_annotated_params += 1
                        if node.returns is not None:
                            total_annotated_returns += 1
            except Exception as e:
                print(f"AST parse error in {path}: {e}")
                continue
        self.api_param_type_stats = {
            'total_functions': total_funcs,
            'total_parameters': total_params,
            'annotated_parameters': total_annotated_params,
            'annotated_returns': total_annotated_returns,
            'param_annotation_coverage': (total_annotated_params / total_params) if total_params else 0,
            'return_annotation_coverage': (total_annotated_returns / total_funcs) if total_funcs else 0
        }

    def get_api_param_type_stats(self):
        # Return function parameter/type annotation statistics
        return getattr(self, 'api_param_type_stats', {})

    def _analyze_file_trends(self, max_points=20):
        # For each file, get line count at each commit (limited to max_points per file)
        git_dir = self.root_dir / '.git'
        if not git_dir.exists():
            self.file_trends = None
            return
        self.file_trends = {}
        for file_stat in self.file_details:
            path = file_stat['path']
            rel_path = os.path.relpath(path, self.root_dir)
            try:
                # Get commit hashes and dates for this file
                cmd = ['git', '-C', str(self.root_dir), 'log', '--format=%H|%ad', '--date=short', '--', rel_path]
                output = subprocess.check_output(cmd, encoding='utf-8', errors='ignore', stderr=subprocess.DEVNULL)
                commits = [line.strip().split('|') for line in output.splitlines() if line.strip()]
                # Limit to latest max_points commits
                commits = commits[:max_points]
                trend = []
                for commit_hash, date in commits:
                    # Get file content at this commit
                    show_cmd = ['git', '-C', str(self.root_dir), 'show', f'{commit_hash}:{rel_path}']
                    try:
                        content = subprocess.check_output(show_cmd, encoding='utf-8', errors='ignore', stderr=subprocess.DEVNULL)
                        line_count = len(content.splitlines())
                    except Exception:
                        line_count = None  # Suppress all git errors
                    trend.append({'commit': commit_hash, 'date': date, 'lines': line_count})
                self.file_trends[path] = trend
            except Exception:
                continue  # Suppress all git errors

    def get_file_trend(self, file, max_points=20):
        # Return line count trend for a file (list of dicts: commit, date, lines)
        if not hasattr(self, 'file_trends') or self.file_trends is None:
            return []
        return self.file_trends.get(file, [])

    def _analyze_refactor_suggestions(self):
        # Mark files/functions as refactor candidates based on metrics
        suggestions = []
        for f in self.file_details:
            reasons = []
            if f.get('complexity', 0) > 20:
                reasons.append('High total complexity')
            if f.get('function_avg_length', 0) > 100:
                reasons.append('Long average function length')
            if f.get('comment_density', 0) < 0.03:
                reasons.append('Low comment density')
            if f.get('todo_count', 0) > 5:
                reasons.append('Many TODO/FIXME')
            if reasons:
                suggestions.append({
                    'path': f['path'],
                    'ext': f['ext'],
                    'total_lines': f['total_lines'],
                    'complexity': f['complexity'],
                    'function_avg_length': f.get('function_avg_length', 0),
                    'comment_density': f.get('comment_density', 0),
                    'todo_count': f.get('todo_count', 0),
                    'reasons': reasons
                })
        self.refactor_suggestions = suggestions

    def get_refactor_suggestions(self):
        # Return list of refactor suggestion dicts
        return getattr(self, 'refactor_suggestions', [])

    def _analyze_openapi(self):
        # Scan for Flask/FastAPI routes and build OpenAPI spec (basic)
        import ast
        openapi = {
            "openapi": "3.0.0",
            "info": {"title": "Auto API", "version": "1.0.0"},
            "paths": {}
        }
        for file_stat in self.file_details:
            if file_stat['ext'] != '.py':
                continue
            path = file_stat['path']
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    tree = ast.parse(f.read(), filename=path)
                for node in ast.walk(tree):
                    # Flask: @app.route('/path', methods=['GET',...])
                    # FastAPI: @app.get('/path'), @app.post('/path')
                    if isinstance(node, ast.FunctionDef) and node.decorator_list:
                        for dec in node.decorator_list:
                            route_path = None
                            methods = []
                            if isinstance(dec, ast.Call) and hasattr(dec.func, 'attr'):
                                # Flask: @app.route
                                if dec.func.attr == 'route' and dec.args:
                                    if isinstance(dec.args[0], ast.Str):
                                        route_path = dec.args[0].s
                                    # methods kwarg
                                    for kw in dec.keywords:
                                        if kw.arg == 'methods' and isinstance(kw.value, ast.List):
                                            methods = [elt.s for elt in kw.value.elts if isinstance(elt, ast.Str)]
                                # FastAPI: @app.get/post/put/delete
                                elif dec.func.attr in ['get', 'post', 'put', 'delete', 'patch'] and dec.args:
                                    if isinstance(dec.args[0], ast.Str):
                                        route_path = dec.args[0].s
                                    methods = [dec.func.attr.upper()]
                            if route_path:
                                if not methods:
                                    methods = ['GET']  # default for Flask
                                for m in methods:
                                    if route_path not in openapi['paths']:
                                        openapi['paths'][route_path] = {}
                                    openapi['paths'][route_path][m.lower()] = {
                                        "summary": node.name,
                                        "description": ast.get_docstring(node) or "",
                                        "responses": {"200": {"description": "Success"}}
                                    }
            except Exception:
                continue
        self.openapi_spec = openapi

    def get_openapi_spec(self):
        # Return OpenAPI 3.0 spec dict
        return getattr(self, 'openapi_spec', None)

    def _analyze_style_issues(self, max_line_length=120):
        # Check for indentation, line length, trailing whitespace, missing newline at EOF
        issues = []
        for file_stat in self.file_details:
            path = file_stat['path']
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    lines = f.readlines()
                for i, line in enumerate(lines):
                    if '\t' in line:
                        issues.append({'file': path, 'line': i+1, 'type': 'tab-indent', 'desc': 'Tab character in indentation'})
                    if len(line.rstrip('\n\r')) > max_line_length:
                        issues.append({'file': path, 'line': i+1, 'type': 'long-line', 'desc': f'Line exceeds {max_line_length} chars'})
                    if line.rstrip('\n\r') != line.rstrip():
                        issues.append({'file': path, 'line': i+1, 'type': 'trailing-whitespace', 'desc': 'Trailing whitespace'})
                if lines and not lines[-1].endswith('\n'):
                    issues.append({'file': path, 'line': len(lines), 'type': 'no-eof-newline', 'desc': 'No newline at end of file'})
            except Exception:
                continue
        self.style_issues = issues

    def get_style_issues(self):
        # Return list of code style issues
        return getattr(self, 'style_issues', [])

    def _analyze_contributor_stats(self):
        # For each file, use git log to count lines/commits per author
        git_dir = self.root_dir / '.git'
        if not git_dir.exists():
            self.contributor_stats = None
            return
        from collections import Counter, defaultdict
        author_files = defaultdict(set)
        author_lines = Counter()
        author_commits = Counter()
        for file_stat in self.file_details:
            path = file_stat['path']
            rel_path = os.path.relpath(path, self.root_dir)
            try:
                # Get all authors for this file
                cmd = ['git', '-C', str(self.root_dir), 'log', '--format=%an', rel_path]
                authors = subprocess.check_output(cmd, encoding='utf-8', errors='ignore', stderr=subprocess.DEVNULL).splitlines()
                for a in set(authors):
                    author_files[a].add(path)
                for a in authors:
                    author_commits[a] += 1
                # Use git blame to count lines per author
                blame_cmd = ['git', '-C', str(self.root_dir), 'blame', '--line-porcelain', rel_path]
                blame_out = subprocess.check_output(blame_cmd, encoding='utf-8', errors='ignore', stderr=subprocess.DEVNULL)
                for line in blame_out.splitlines():
                    if line.startswith('author '):
                        author = line[7:]
                        author_lines[author] += 1
            except Exception:
                continue  # Suppress all git errors
        stats = []
        for author in set(list(author_files.keys()) + list(author_lines.keys()) + list(author_commits.keys())):
            stats.append({
                'author': author,
                'file_count': len(author_files[author]),
                'line_count': author_lines[author],
                'commit_count': author_commits[author]
            })
        self.contributor_stats = stats

    def get_contributor_stats(self):
        # Return list of contributor stats dicts
        return getattr(self, 'contributor_stats', [])

    def _analyze_advanced_security_issues(self):
        # Scan for advanced security issues: SSRF, RCE, SQLi, secrets
        import re
        patterns = [
            (r'requests\.get\s*\(\s*input\(', 'Potential SSRF: requests.get(input())'),
            (r'os\.system\s*\(', 'Potential RCE: os.system()'),
            (r'subprocess\.Popen\s*\(', 'Potential RCE: subprocess.Popen()'),
            (r'\bexec\s*\(', 'Potential RCE: exec()'),
            (r'\beval\s*\(', 'Potential RCE: eval()'),
            (r'\bSELECT\b.*\bFROM\b.*\+.*input\(', 'Potential SQLi: dynamic SQL with input()'),
            (r'aws_secret_access_key\s*=\s*["\"][^"\"]+["\"]', 'Hardcoded AWS secret'),
            (r'aws_access_key_id\s*=\s*["\"][^"\"]+["\"]', 'Hardcoded AWS key'),
            (r'AIza[0-9A-Za-z\-_]{35}', 'Hardcoded Google API key'),
            (r'slack_token\s*=\s*["\"][^"\"]+["\"]', 'Hardcoded Slack token'),
            (r'github_pat_[0-9a-zA-Z_]{22,255}', 'Hardcoded GitHub token'),
        ]
        issues = []
        for file_stat in self.file_details:
            path = file_stat['path']
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    for lineno, line in enumerate(f, 1):
                        for pat, desc in patterns:
                            if re.search(pat, line, re.IGNORECASE):
                                issues.append({'file': path, 'line': lineno, 'desc': desc, 'content': line.strip(), 'note': 'Potential high-risk security issue'})
            except Exception:
                continue
        self.advanced_security_issues = issues

    def get_advanced_security_issues(self):
        # Return list of advanced security issues
        return getattr(self, 'advanced_security_issues', [])

    def _iter_files(self, root):
        # Generator that yields files, skipping excluded directories
        for dirpath, dirnames, filenames in os.walk(root):
            # Remove excluded directories in-place
            dirnames[:] = [d for d in dirnames if d not in self.exclude_dirs]
            for filename in filenames:
                yield pathlib.Path(dirpath) / filename

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

    def get_file_details_with_size(self):
        # Return per-file statistics including file size
        return self.file_details 