#!/usr/bin/env nu

# Incus Composer Documentation Build Script
# This script builds the documentation using AsciiDoctor

# Global configuration
let book_dir = ($env.FILE_PWD)
let output_dir = ($book_dir | path join "output")
let source_file = ($book_dir | path join "index.adoc")

# Color codes for output
let colors = {
    red: (ansi red),
    green: (ansi green),
    yellow: (ansi yellow),
    blue: (ansi blue),
    reset: (ansi reset)
}

# Print colored messages
def print_info [message: string] {
    print $"($colors.blue)[INFO]($colors.reset) ($message)"
}

def print_success [message: string] {
    print $"($colors.green)[SUCCESS]($colors.reset) ($message)"
}

def print_warning [message: string] {
    print $"($colors.yellow)[WARNING]($colors.reset) ($message)"
}

def print_error [message: string] {
    print $"($colors.red)[ERROR]($colors.reset) ($message)"
}

# Check for required dependencies
def check_dependencies [] {
    print_info "Checking dependencies..."

    if not (which asciidoctor | is-not-empty) {
        print_error "asciidoctor not found. Please install it:"
        print "  Ubuntu/Debian: sudo apt install asciidoctor"
        print "  macOS: brew install asciidoctor"
        print "  Or via gem: gem install asciidoctor"
        exit 1
    }

    if not (which asciidoctor-pdf | is-not-empty) {
        print_warning "asciidoctor-pdf not found. PDF generation will be skipped."
        print "  Install with: gem install asciidoctor-pdf"
    }

    print_success "Dependencies check completed"
}

# Create output directory
def create_output_dir [] {
    print_info "Creating output directory..."
    mkdir $output_dir
    print_success $"Output directory created: ($output_dir)"
}

# Build HTML documentation
def build_html [] {
    print_info "Building HTML documentation..."

    let result = (run-external "asciidoctor"
        "--destination-dir" $output_dir
        "--out-file" "index.html"
        "--attribute" "toc=left"
        "--attribute" "toclevels=3"
        "--attribute" "sectlinks"
        "--attribute" "sectanchors"
        "--attribute" "source-highlighter=rouge"
        "--attribute" "icons=font"
        "--attribute" "stylesheet"
        $source_file
    )

    print_success $"HTML documentation built: ($output_dir | path join 'index.html')"
}

# Build PDF documentation
def build_pdf [] {
    if (which asciidoctor-pdf | is-not-empty) {
        print_info "Building PDF documentation..."

        let result = (run-external "asciidoctor-pdf"
            "--destination-dir" $output_dir
            "--out-file" "incus-composer.pdf"
            "--attribute" "pdf-theme=default"
            "--attribute" "pdf-fontsdir=GEM_FONTS_DIR"
            $source_file
        )

        print_success $"PDF documentation built: ($output_dir | path join 'incus-composer.pdf')"
    } else {
        print_warning "Skipping PDF generation (asciidoctor-pdf not available)"
    }
}

# Build DocBook XML
def build_docbook [] {
    print_info "Building DocBook XML..."

    let result = (run-external "asciidoctor"
        "--backend" "docbook"
        "--destination-dir" $output_dir
        "--out-file" "incus-composer.xml"
        $source_file
    )

    print_success $"DocBook XML built: ($output_dir | path join 'incus-composer.xml')"
}

# Copy assets if they exist
def copy_assets [] {
    print_info "Copying assets..."

    let images_dir = ($book_dir | path join "images")
    if ($images_dir | path exists) {
        cp -r $images_dir $output_dir
        print_success "Images copied to output directory"
    }

    let custom_css = ($book_dir | path join "custom.css")
    if ($custom_css | path exists) {
        cp $custom_css $output_dir
        print_success "Custom CSS copied to output directory"
    }
}

# Show usage information
def show_usage [] {
    print "Usage: build.nu [OPTIONS]"
    print ""
    print "Build the Incus Composer documentation"
    print ""
    print "OPTIONS:"
    print "  -h, --help     Show this help message"
    print "  -c, --clean    Clean output directory before building"
    print "  --html-only    Build only HTML output"
    print "  --pdf-only     Build only PDF output"
    print "  --no-pdf       Skip PDF generation"
    print ""
    print "Examples:"
    print "  nu build.nu                # Build all formats"
    print "  nu build.nu --clean       # Clean and build all formats"
    print "  nu build.nu --html-only   # Build only HTML"
    print "  nu build.nu --no-pdf      # Build all except PDF"
}

# Clean output directory
def clean_output [] {
    print_info "Cleaning output directory..."
    if ($output_dir | path exists) {
        rm -rf $output_dir
    }
    print_success "Output directory cleaned"
}

# List generated files
def list_generated_files [] {
    print_info "Generated files:"
    if ($output_dir | path exists) {
        ls $output_dir
        | get name
        | each { |file| $"  - (($file | path basename))" }
        | str join "\n"
        | print
    }
}

# Main build function
def main [
    --help (-h)               # Show help message
    --clean (-c)              # Clean output directory before building
    --html-only               # Build only HTML output
    --pdf-only                # Build only PDF output
    --no-pdf                  # Skip PDF generation
] {

    # Show help if requested
    if $help {
        show_usage
        exit 0
    }

    # Validate conflicting options
    if $html_only and $pdf_only {
        print_error "Cannot specify both --html-only and --pdf-only"
        exit 1
    }

    print_info "Starting documentation build process..."

    # Clean output directory if requested
    if $clean {
        clean_output
    }

    check_dependencies
    create_output_dir

    # Build based on options
    if $pdf_only {
        build_pdf
    } else if $html_only {
        build_html
        build_docbook
        copy_assets
    } else {
        build_html
        build_docbook
        copy_assets

        if not $no_pdf {
            build_pdf
        }
    }

    print_success "Documentation build completed!"
    print_info $"Output directory: ($output_dir)"

    # List generated files
    list_generated_files
}
