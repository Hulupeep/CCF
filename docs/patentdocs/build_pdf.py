#!/usr/bin/env python3
"""
Build USPTO provisional patent application PDF from markdown + SVG drawings.

Requirements: weasyprint, cairosvg, markdown (all installed)
Usage: python3 build_pdf.py
"""

import re
import base64
import os
import sys
from pathlib import Path
import markdown as md_lib

PATENT_MD   = Path(__file__).parent / "patent.md"
DRAWINGS_DIR = Path(__file__).parent / "files" / "drawings"
OUTPUT_PDF   = Path(__file__).parent / "ccf-provisional-filing.pdf"

# Maps figure numbers to SVG filenames
FIGURE_FILES = {
    1:  "fig01_system_architecture.svg",
    2:  "fig02_phase_space.svg",
    3:  "fig03_per_tick_flow.svg",
    4:  "fig04_mincut_matrix.svg",
    5:  "fig05_coherence_dynamics.svg",
    6:  "fig06_bidirectional.svg",
    7:  "fig07_compilation.svg",
    8:  "fig08_consolidation.svg",
    9:  "fig09_conflict.svg",
    10: "fig10_mixing_architecture.svg",
    11: "fig11_matrix_evolution.svg",
}

FIGURE_CAPTIONS = {
    1:  "FIG. 1 — Overall system architecture of the Contextual Coherence Fields system with dual-process cognitive architecture.",
    2:  "FIG. 2 — Two-dimensional behavioral phase space: four quadrants defined by effective coherence and tension axes.",
    3:  "FIG. 3 — Per-tick processing cycle of the reflexive processing unit.",
    4:  "FIG. 4 — Relational graph construction and graph min-cut for context boundary discovery.",
    5:  "FIG. 5 — Coherence accumulator dynamics: asymptotic growth, earned floor, and recovery from negative events.",
    6:  "FIG. 6 — Bidirectional modulation pathways of the dual-process cognitive architecture.",
    7:  "FIG. 7 — Habit compilation lifecycle: deliberative processing to compiled reflexive execution.",
    8:  "FIG. 8 — Consolidation cycle timeline during idle period.",
    9:  "FIG. 9 — Classification conflict resolution: hesitation trace and resolution pathway.",
    10: "FIG. 10 — Manifold-constrained coherence mixing architecture and Birkhoff polytope projection.",
    11: "FIG. 11 — Evolution of the mixing matrix over the operational life of the robot.",
}


def svg_to_data_uri(svg_path: Path) -> str:
    """Read SVG and return a data URI for embedding in HTML."""
    svg_text = svg_path.read_text(encoding="utf-8")
    # Inline SVG is most reliable with weasyprint; return as-is for wrapping
    return svg_text


def figure_html(fig_num: int) -> str:
    """Build a figure block for insertion into the HTML body."""
    svg_path = DRAWINGS_DIR / FIGURE_FILES[fig_num]
    if not svg_path.exists():
        return f'<div class="figure"><p class="fig-caption">[Figure {fig_num} — file not found]</p></div>'

    # Embed SVG inline (weasyprint handles this well)
    svg_text = svg_path.read_text(encoding="utf-8")

    # Strip XML declaration and DOCTYPE if present
    svg_text = re.sub(r'<\?xml[^>]*\?>', '', svg_text).strip()
    svg_text = re.sub(r'<!DOCTYPE[^>]*>', '', svg_text).strip()

    caption = FIGURE_CAPTIONS.get(fig_num, f"FIG. {fig_num}")

    return f'''<div class="figure">
<div class="figure-svg">{svg_text}</div>
<p class="fig-caption">{caption}</p>
</div>'''


def preprocess_markdown(text: str) -> str:
    """Apply required text substitutions to the markdown source."""

    # 1. Update filing date
    text = text.replace(
        "**Filing Date: [TO BE INSERTED]**",
        "**Filing Date: February 23, 2026**"
    )

    # 2. Remove CONFIDENTIAL line (and surrounding blank lines)
    text = re.sub(r'\n\n\*\*CONFIDENTIAL\*\*\n\n', '\n\n', text)
    text = re.sub(r'\*\*CONFIDENTIAL\*\*\n?', '', text)

    return text


def insert_figure_markers(text: str) -> str:
    """
    Insert {{FIG_N}} markers into the markdown text immediately after
    the paragraph that first says "Referring now to FIG. N" or "FIG. N".

    For figures not explicitly referenced in the body (FIG. 3, FIG. 5),
    insert them after the BRIEF DESCRIPTION OF THE DRAWINGS section.
    """
    paragraphs = text.split('\n\n')
    result = []
    inserted = set()

    # Track position of "BRIEF DESCRIPTION" section for orphaned figures
    brief_desc_idx = None

    for i, para in enumerate(paragraphs):
        result.append(para)

        if 'BRIEF DESCRIPTION OF THE DRAWINGS' in para:
            brief_desc_idx = i

        # Only insert figures when the detailed description says "Referring now to FIG. N"
        # Do NOT trigger on the Brief Description mentions of "FIG. N"
        refs = re.findall(r'Referring now to FIG\.\s*(\d+)', para)
        for ref_str in refs:
            fig_num = int(ref_str)
            if fig_num in FIGURE_FILES and fig_num not in inserted:
                result.append(f"{{{{FIG_{fig_num}}}}}")
                inserted.add(fig_num)

    # Insert orphaned figures (3 and 5 are described in Brief Desc but not
    # referenced with "Referring now to" in the body).
    # We'll place them right after the last figure in the Brief Desc block,
    # i.e. after we've processed Brief Desc — find the note paragraph:
    # "[Note: Formal drawings to be prepared...]" follows [0029].
    # Easiest: find that note and insert after it.
    out_text = '\n\n'.join(result)

    # After the note about formal drawings, insert remaining figures in order
    note_pattern = r'(\*\[Note: Formal drawings to be prepared[^\]]*\]\*)'
    match = re.search(note_pattern, out_text)
    if match:
        insertion_point = match.end()
        orphan_blocks = []
        for fig_num in sorted(FIGURE_FILES.keys()):
            if fig_num not in inserted:
                orphan_blocks.append(f"\n\n{{{{FIG_{fig_num}}}}}")
                inserted.add(fig_num)
        if orphan_blocks:
            out_text = out_text[:insertion_point] + ''.join(orphan_blocks) + out_text[insertion_point:]

    return out_text


def markers_to_html(text: str) -> str:
    """Replace {{FIG_N}} markers with the actual figure HTML."""
    def replacer(m):
        fig_num = int(m.group(1))
        return figure_html(fig_num)

    # We need to do this AFTER markdown conversion, so for now just note positions
    # by converting markers to a unique placeholder that markdown won't mangle.
    def marker_to_placeholder(m):
        return f'\n\nFIGUREPLACEHOLDER{m.group(1)}\n\n'

    text = re.sub(r'\{\{FIG_(\d+)\}\}', marker_to_placeholder, text)
    return text


def convert_html_figures(html: str) -> str:
    """Replace FIGUREPLACEHOLDERN text (as it comes out of markdown) with actual figure HTML."""
    def replacer(m):
        fig_num = int(m.group(1))
        return figure_html(fig_num)

    # markdown wraps the placeholder in a <p> tag, handle both cases
    html = re.sub(r'<p>FIGUREPLACEHOLDER(\d+)</p>', replacer, html)
    html = re.sub(r'FIGUREPLACEHOLDER(\d+)', replacer, html)
    return html


CSS = """
@page {
    size: letter;
    margin: 1in;
    @bottom-center {
        content: counter(page);
        font-family: 'Times New Roman', Times, Georgia, serif;
        font-size: 10pt;
    }
}

body {
    font-family: 'Times New Roman', Times, Georgia, serif;
    font-size: 12pt;
    line-height: 1.5;
    color: #000000;
    text-align: justify;
    hyphens: auto;
}

h1 {
    font-size: 13pt;
    font-weight: bold;
    text-align: center;
    text-transform: uppercase;
    margin-top: 1em;
    margin-bottom: 0.5em;
    page-break-before: avoid;
}

h2 {
    font-size: 12pt;
    font-weight: bold;
    text-align: center;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
}

h3 {
    font-size: 12pt;
    font-weight: bold;
    margin-top: 1.2em;
    margin-bottom: 0.3em;
}

h4 {
    font-size: 12pt;
    font-weight: bold;
    font-style: italic;
    margin-top: 1em;
    margin-bottom: 0.3em;
}

p {
    margin-top: 0;
    margin-bottom: 0.6em;
    text-indent: 0;
}

strong {
    font-weight: bold;
}

em {
    font-style: italic;
}

/* Paragraph number labels like [0001] */
strong > strong, p > strong:first-child {
    font-weight: bold;
}

ul, ol {
    margin-left: 2em;
    margin-bottom: 0.6em;
}

li {
    margin-bottom: 0.3em;
}

hr {
    border: none;
    border-top: 1px solid #000;
    margin: 1.5em 0;
}

/* Title block */
.title-block {
    text-align: center;
    margin-bottom: 2em;
}

/* Figure blocks — each on its own page */
.figure {
    page-break-before: always;
    page-break-after: always;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 6in;
    margin: 0;
    padding: 0.5in 0;
}

.figure-svg {
    width: 100%;
    max-width: 6.5in;
    display: flex;
    justify-content: center;
    align-items: center;
}

.figure-svg svg {
    width: 100%;
    height: auto;
    max-height: 7.5in;
}

.fig-caption {
    text-align: center;
    font-size: 10pt;
    font-style: italic;
    margin-top: 0.5em;
    text-indent: 0;
}

/* Claims section — keep numbered items together */
.claims p {
    page-break-inside: avoid;
}

/* References */
blockquote {
    margin-left: 2em;
}

/* Code blocks (shouldn't be many) */
code, pre {
    font-family: 'Courier New', Courier, monospace;
    font-size: 10pt;
}

/* Abstract — last section, minimal indentation */
h2 + p {
    text-indent: 0;
}
"""


FULL_HTML_TEMPLATE = """\
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<style>
{css}
</style>
</head>
<body>
{body}
</body>
</html>
"""


def build_html(md_text: str) -> str:
    """Convert processed markdown to a full HTML document."""

    # 1. Apply text substitutions
    md_text = preprocess_markdown(md_text)

    # 2. Insert figure markers into markdown
    md_text = insert_figure_markers(md_text)

    # 3. Replace {{FIG_N}} with unique placeholders that survive markdown conversion
    md_text = markers_to_html(md_text)

    # 4. Convert markdown to HTML
    html_body = md_lib.markdown(
        md_text,
        extensions=['extra', 'sane_lists'],
    )

    # 5. Replace figure placeholders with actual figure HTML
    html_body = convert_html_figures(html_body)

    # 6. Wrap in full document
    return FULL_HTML_TEMPLATE.format(css=CSS, body=html_body)


def main():
    print("Reading patent markdown...")
    md_text = PATENT_MD.read_text(encoding="utf-8")

    print("Building HTML...")
    html = build_html(md_text)

    # Write intermediate HTML for debugging if needed
    html_out = OUTPUT_PDF.with_suffix('.html')
    html_out.write_text(html, encoding="utf-8")
    print(f"Intermediate HTML written to: {html_out}")

    print("Converting to PDF with weasyprint...")
    try:
        from weasyprint import HTML, CSS as WpCSS
        from weasyprint.text.fonts import FontConfiguration

        font_config = FontConfiguration()
        wp_html = HTML(string=html, base_url=str(DRAWINGS_DIR))
        wp_html.write_pdf(
            str(OUTPUT_PDF),
            font_config=font_config,
        )
        print(f"\nDone! PDF written to:\n  {OUTPUT_PDF}")
        size_kb = OUTPUT_PDF.stat().st_size // 1024
        print(f"  Size: {size_kb} KB")
    except Exception as e:
        print(f"Error during PDF generation: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
