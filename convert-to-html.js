const fs = require('fs');
const { marked } = require('marked');

// Configure marked to preserve mermaid code blocks
marked.setOptions({
  highlight: function(code, lang) {
    if (lang === 'mermaid') {
      return `<div class="mermaid">${code}</div>`;
    }
    return `<pre><code class="language-${lang}">${code}</code></pre>`;
  }
});

// Read the markdown file
const markdownContent = fs.readFileSync('SOCIAL_PULSE_ARCHITECTURE_DOCUMENTATION.md', 'utf8');

// Convert markdown to HTML
let htmlContent = marked(markdownContent);

// Post-process to convert remaining mermaid code blocks to divs
htmlContent = htmlContent.replace(
  /<pre><code class="language-mermaid">([\s\S]*?)<\/code><\/pre>/g,
  '<div class="mermaid">$1</div>'
);

// Create a complete HTML document with styling
const fullHTML = `
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Social Pulse - Architecture Documentation</title>
    <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
    <script>
        document.addEventListener('DOMContentLoaded', function() {
            mermaid.initialize({ 
                startOnLoad: true,
                theme: 'default',
                flowchart: {
                    useMaxWidth: true,
                    htmlLabels: true
                }
            });
        });
    </script>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Open Sans', 'Helvetica Neue', sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 900px;
            margin: 0 auto;
            padding: 2rem;
            background: white;
        }
        
        .mermaid {
            text-align: center;
            margin: 2rem 0;
            page-break-inside: avoid;
            max-width: 100%;
            overflow: auto;
        }
        
        h1 {
            color: #2c3e50;
            border-bottom: 3px solid #3498db;
            padding-bottom: 0.5rem;
            margin-top: 2rem;
            page-break-before: always;
        }
        
        h1:first-child {
            page-break-before: auto;
        }
        
        h2 {
            color: #34495e;
            border-bottom: 2px solid #ecf0f1;
            padding-bottom: 0.3rem;
            margin-top: 1.5rem;
        }
        
        h3 {
            color: #34495e;
            margin-top: 1.2rem;
        }
        
        code {
            background-color: #f8f9fa;
            padding: 0.2rem 0.4rem;
            border-radius: 3px;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            color: #e74c3c;
        }
        
        pre {
            background-color: #f8f9fa;
            border: 1px solid #e9ecef;
            border-radius: 5px;
            padding: 1rem;
            overflow-x: auto;
            page-break-inside: avoid;
        }
        
        pre code {
            background: none;
            color: #333;
        }
        
        table {
            border-collapse: collapse;
            width: 100%;
            margin: 1rem 0;
            page-break-inside: avoid;
        }
        
        th, td {
            border: 1px solid #ddd;
            padding: 0.7rem;
            text-align: left;
        }
        
        th {
            background-color: #f8f9fa;
            font-weight: 600;
        }
        
        blockquote {
            border-left: 4px solid #3498db;
            padding-left: 1rem;
            margin: 1rem 0;
            background-color: #f8f9fa;
            padding: 1rem;
            page-break-inside: avoid;
        }
        
        .mermaid-note {
            background-color: #fff3cd;
            border: 1px solid #ffeaa7;
            border-radius: 5px;
            padding: 1rem;
            margin: 1rem 0;
            color: #856404;
            page-break-inside: avoid;
        }
        
        ul, ol {
            padding-left: 2rem;
        }
        
        li {
            margin: 0.3rem 0;
        }
        
        .pdf-instructions {
            background-color: #e3f2fd;
            border: 1px solid #2196f3;
            border-radius: 5px;
            padding: 1rem;
            margin: 2rem 0;
            color: #1565c0;
            text-align: center;
        }
        
        .conversion-steps {
            background-color: #f8f9fa;
            border: 1px solid #dee2e6;
            border-radius: 5px;
            padding: 1.5rem;
            margin: 2rem 0;
            text-align: left;
        }
        
        .conversion-steps h4 {
            color: #495057;
            margin-top: 0;
        }
        
        .conversion-steps ol {
            margin: 0.5rem 0;
        }
        
        .conversion-steps li {
            margin: 0.5rem 0;
        }
        
        @media print {
            body {
                max-width: none;
                margin: 0;
                padding: 1rem;
            }
            
            .pdf-instructions {
                display: none;
            }
            
            h1, h2, h3 {
                page-break-after: avoid;
            }
        }
        
        @page {
            margin: 1in;
        }
    </style>
</head>
<body>
    <div class="pdf-instructions">
        <strong>📄 PDF Conversion Instructions:</strong><br>
        This HTML file is ready for PDF conversion with rendered Mermaid diagrams!
    </div>
    
    <div class="conversion-steps">
        <h4>🖨️ Recommended PDF Conversion Methods:</h4>
        <ol>
            <li><strong>Browser Print (Recommended):</strong>
                <ul>
                    <li>Open this file in Chrome or Firefox</li>
                    <li>Wait for diagrams to fully render (2-3 seconds)</li>
                    <li>Press Ctrl+P (Cmd+P on Mac) or go to File → Print</li>
                    <li>Select "Save as PDF" as destination</li>
                    <li>Set paper size to A4 or Letter</li>
                    <li>Enable "Background graphics"</li>
                    <li>Set margins to "Default" or "Minimum"</li>
                    <li>Click "Save"</li>
                </ul>
            </li>
            <li><strong>Online Converters:</strong>
                <ul>
                    <li><a href="https://www.ilovepdf.com/html-to-pdf">ilovepdf.com/html-to-pdf</a></li>
                    <li><a href="https://pdfcrowd.com/html-to-pdf/">pdfcrowd.com/html-to-pdf</a></li>
                </ul>
            </li>
            <li><strong>Professional Tools:</strong>
                <ul>
                    <li>Typora: Open the .md file → File → Export → PDF</li>
                    <li>VS Code: Install "Markdown PDF" extension</li>
                </ul>
            </li>
        </ol>
    </div>
    
    ${htmlContent}
    
    <div class="mermaid-note">
        <strong>📊 Note about Diagrams:</strong> This document references Mermaid diagrams in the source Markdown. 
        The textual descriptions provide complete architectural information. For visual diagrams, the source 
        Markdown can be viewed in tools like Typora, GitLab, or GitHub that support Mermaid rendering.
    </div>
</body>
</html>`;

// Save the HTML file
fs.writeFileSync('Social_Pulse_Architecture_Documentation.html', fullHTML);

console.log('✅ HTML file generated successfully: Social_Pulse_Architecture_Documentation.html');
console.log('💡 Open this file in your browser and print to PDF, or use online conversion tools.');