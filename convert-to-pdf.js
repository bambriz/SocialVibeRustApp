const fs = require('fs');
const { marked } = require('marked');
const puppeteer = require('puppeteer');

async function convertMarkdownToPDF() {
    try {
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
        }
        
        pre code {
            background: none;
            color: #333;
        }
        
        table {
            border-collapse: collapse;
            width: 100%;
            margin: 1rem 0;
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
        }
        
        .mermaid-note {
            background-color: #fff3cd;
            border: 1px solid #ffeaa7;
            border-radius: 5px;
            padding: 1rem;
            margin: 1rem 0;
            color: #856404;
        }
        
        ul, ol {
            padding-left: 2rem;
        }
        
        li {
            margin: 0.3rem 0;
        }
        
        .page-break {
            page-break-before: always;
        }
        
        @media print {
            body {
                max-width: none;
                margin: 0;
                padding: 1rem;
            }
            
            h1 {
                page-break-before: always;
            }
            
            h1:first-child {
                page-break-before: auto;
            }
        }
    </style>
</head>
<body>
    ${htmlContent}
    
    <div class="mermaid-note">
        <strong>Note:</strong> This PDF was generated from Markdown. Mermaid diagrams referenced in the source document 
        would require additional rendering tools for full visualization. The textual descriptions provide the complete 
        architectural information.
    </div>
</body>
</html>`;
        
        // Launch puppeteer
        console.log('Launching browser...');
        const browser = await puppeteer.launch({
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });
        
        const page = await browser.newPage();
        
        // Set content and generate PDF
        console.log('Converting to PDF...');
        await page.setContent(fullHTML, { waitUntil: 'networkidle0' });
        
        const pdf = await page.pdf({
            format: 'A4',
            printBackground: true,
            margin: {
                top: '1in',
                right: '1in',
                bottom: '1in',
                left: '1in'
            },
            displayHeaderFooter: true,
            headerTemplate: '<div style="font-size: 10px; text-align: center; width: 100%;"><span class="title">Social Pulse - Architecture Documentation</span></div>',
            footerTemplate: '<div style="font-size: 10px; text-align: center; width: 100%;"><span class="pageNumber"></span> / <span class="totalPages"></span></div>'
        });
        
        // Save the PDF
        fs.writeFileSync('Social_Pulse_Architecture_Documentation.pdf', pdf);
        
        await browser.close();
        
        console.log('✅ PDF generated successfully: Social_Pulse_Architecture_Documentation.pdf');
        
    } catch (error) {
        console.error('❌ Error generating PDF:', error);
        process.exit(1);
    }
}

convertMarkdownToPDF();