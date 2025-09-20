const fs = require('fs');
const { marked } = require('marked');

// Read the markdown file
const markdownContent = fs.readFileSync('SOCIAL_PULSE_ARCHITECTURE_DOCUMENTATION.md', 'utf8');

// Convert markdown to HTML
const htmlContent = marked(markdownContent);

// Create a complete HTML document with styling
const fullHTML = `
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Social Pulse - Architecture Documentation</title>
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
        To convert this HTML file to PDF: <br>
        1. Open this file in Chrome/Firefox → Print → Save as PDF<br>
        2. Or use online tools like <a href="https://html-pdf-api.netlify.app/">html-pdf-api.netlify.app</a><br>
        3. For best results, use A4 page size with 1-inch margins
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