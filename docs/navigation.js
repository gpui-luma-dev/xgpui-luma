// Shared desktop and mobile navigation. No fetch or build step; works on file://.
(() => {
    const sections = [
    {
        "title": "Overview",
        "items": [
            {
                "href": "index.html",
                "label": "Getting Started"
            },
            {
                "href": "motion.html",
                "label": "Motion"
            },
            {
                "href": "control-specialization.html",
                "label": "Create a control specialization"
            },
            {
                "href": "xgpui-lab.html",
                "label": "XGPUI Lab App",
                "children": [
                    {
                        "href": "xgpui-lab-screen-design.html",
                        "label": "Lab Screen Design"
                    }
                ]
            }
        ]
    },
    {
        "title": "Looks",
        "items": [
            {
                "href": "looks.html",
                "label": "Lookless"
            },
            {
                "href": "look-shadcn.html",
                "label": "Look-shadcn"
            },
            {
                "href": "look-radix.html",
                "label": "Look-radix"
            },
            {
                "href": "create-look.html",
                "label": "Create your own Look"
            }
        ]
    },
    {
        "title": "Controls",
        "items": [
            {
                "href": "accordion.html",
                "label": "Accordion"
            },
            {
                "href": "autocomplete.html",
                "label": "Autocomplete"
            },
            {
                "href": "button.html",
                "label": "Button"
            },
            {
                "href": "checkbox.html",
                "label": "Checkbox"
            },
            {
                "href": "combobox.html",
                "label": "ComboBox"
            },
            {
                "href": "context-menu.html",
                "label": "Context Menu"
            },
            {
                "href": "icon-button.html",
                "label": "Icon Button"
            },
            {
                "href": "icon-group.html",
                "label": "Icon Group"
            },
            {
                "href": "listbox.html",
                "label": "List Box"
            },
            {
                "href": "pager.html",
                "label": "Pager"
            },
            {
                "href": "popover-button.html",
                "label": "Popover Button"
            },
            {
                "href": "popup-menu.html",
                "label": "Popup Menu"
            },
            {
                "href": "progress.html",
                "label": "Progress"
            },
            {
                "href": "radio-button.html",
                "label": "Radio Button"
            },
            {
                "href": "radio-group.html",
                "label": "Radio Group"
            },
            {
                "href": "scrollbar.html",
                "label": "Scrollbar"
            },
            {
                "href": "search-selector.html",
                "label": "Search Selector"
            },
            {
                "href": "selection-panel.html",
                "label": "Selection Panel"
            },
            {
                "href": "selector.html",
                "label": "Selector"
            },
            {
                "href": "sidebar.html",
                "label": "Sidebar"
            },
            {
                "href": "slide-panel.html",
                "label": "Slide Panel"
            },
            {
                "href": "slider.html",
                "label": "Slider"
            },
            {
                "href": "split-button.html",
                "label": "Split Button"
            },
            {
                "href": "stepper.html",
                "label": "Stepper"
            },
            {
                "href": "switch.html",
                "label": "Switch"
            },
            {
                "href": "table.html",
                "label": "Table"
            },
            {
                "href": "tabs.html",
                "label": "Tabs"
            },
            {
                "href": "textarea.html",
                "label": "Text Area"
            },
            {
                "href": "textfield.html",
                "label": "Text Field"
            },
            {
                "href": "toggle.html",
                "label": "Toggle"
            },
            {
                "href": "toolbar.html",
                "label": "Toolbar"
            },
            {
                "href": "tooltip.html",
                "label": "Tooltip"
            },
            {
                "href": "tree-view.html",
                "label": "Tree View"
            }
        ]
    },
    {
        "title": "Layout & Containers",
        "items": [
            {
                "href": "layout.html",
                "label": "Primitives & macros"
            },
            {
                "href": "control-group.html",
                "label": "Control Group"
            },
            {
                "href": "frame.html",
                "label": "Frame"
            },
            {
                "href": "overlay-window.html",
                "label": "Overlay Window"
            },
            {
                "href": "scroll-container.html",
                "label": "Scroll Container"
            },
            {
                "href": "dock-splitter.html",
                "label": "Dock Splitter"
            },
            {
                "href": "resizable-panels.html",
                "label": "Resizable Panels"
            },
            {
                "href": "split-view.html",
                "label": "Split View"
            }
        ]
    },
    {
        "title": "Color controls",
        "items": [
            {
                "href": "colors.html",
                "label": "Overview"
            },
            {
                "href": "color-slider.html",
                "label": "Color Slider"
            },
            {
                "href": "color-field.html",
                "label": "Color Field"
            },
            {
                "href": "color-ring.html",
                "label": "Color Ring"
            },
            {
                "href": "color-arc.html",
                "label": "Color Arc"
            },
            {
                "href": "color-swatch.html",
                "label": "Color Swatch"
            }
        ]
    },
    {
        "title": "Composition & helpers",
        "items": [
            {
                "href": "composition.html",
                "label": "Overview & forms"
            },
            {
                "href": "drag-drop.html",
                "label": "Drag and drop"
            },
            {
                "href": "floating-menu.html",
                "label": "Floating Menu"
            },
            {
                "href": "popup-scroll-surface.html",
                "label": "Popup Scroll Surface"
            },
            {
                "href": "selector-list.html",
                "label": "Selector List"
            }
        ]
    },
    {
        "title": "Plots",
        "items": [
            {
                "href": "plots.html",
                "label": "Getting started"
            },
            {
                "href": "plot-cartesian.html",
                "label": "Cartesian charts"
            },
            {
                "href": "plot-pie.html",
                "label": "Pie & donut"
            },
            {
                "href": "plot-radar.html",
                "label": "Radar charts"
            },
            {
                "href": "plot-radial.html",
                "label": "Radial charts"
            },
            {
                "href": "plot-styling.html",
                "label": "Styling"
            },
            {
                "href": "plot-interaction.html",
                "label": "Interaction"
            },
            {
                "href": "plot-view-state.html",
                "label": "View state"
            },
            {
                "href": "plot-performance.html",
                "label": "Performance"
            }
        ]
    }
];
    const currentPage = window.location.pathname.split('/').pop() || 'index.html';

    function createLink(item, className) {
        const link = document.createElement('a');
        link.setAttribute('href', item.href);
        link.textContent = item.label;
        if (className) link.className = className;
        if (item.href === currentPage) link.setAttribute('aria-current', 'page');
        return link;
    }

    // An item may list child pages, shown indented under it.
    const createLinks = item => [createLink(item), ...(item.children || []).map(child => createLink(child, 'nav-sub'))];

    const sidebar = document.querySelector('[data-site-sidebar]');
    if (sidebar) {
        sidebar.replaceChildren(...sections.map(section => {
            const nav = document.createElement('nav');
            nav.setAttribute('aria-label', section.title);
            const label = document.createElement('div');
            label.className = 'nav-label';
            label.textContent = section.title;
            nav.append(label, ...section.items.flatMap(createLinks));
            return nav;
        }));
    }

    const mobile = document.querySelector('[data-site-mobile-nav]');
    if (mobile) {
        const details = document.createElement('details');
        details.className = 'mobile-nav';
        const summary = document.createElement('summary');
        summary.textContent = 'Documentation navigation';
        const nav = document.createElement('nav');
        nav.setAttribute('aria-label', 'Mobile navigation');
        nav.append(...sections.map(section => {
            const group = document.createElement('div');
            group.className = 'mobile-nav-group';
            const label = document.createElement('strong');
            label.textContent = section.title;
            group.append(label, ...section.items.flatMap(createLinks));
            return group;
        }));
        details.append(summary, nav);
        mobile.replaceChildren(details);
    }
})();
