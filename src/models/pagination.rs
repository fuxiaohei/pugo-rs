use crate::models;

#[derive(Debug)]
pub struct PaginationItem {
    pub current: usize,
    pub total: usize,
    pub total_pages: usize,
    pub size: usize,
    pub has_previous: bool,
    pub has_next: bool,
    pub previous: usize,
    pub next: usize,
    pub start: usize,
    pub end: usize,
    layout: String,
}

impl PaginationItem {
    pub fn new(
        mut current: usize,
        total: usize,
        total_pages: usize,
        size: usize,
        link: &str,
    ) -> PaginationItem {
        if current < 1 {
            current = 1;
        }
        let start = (current - 1) * size;
        let mut end = start + size;
        if end > total {
            end = total;
        }
        let has_previous = current > 1;
        let previous = if has_previous { current - 1 } else { 1 };
        let has_next = end < total;
        let next = if has_next { current + 1 } else { current };

        PaginationItem {
            current,
            total,
            total_pages,
            size,
            has_previous,
            has_next,
            previous,
            next,
            start,
            end,
            layout: link.to_string(),
        }
    }
    fn build_url(&self, page: usize) -> String {
        self.layout.clone().replace(":page", &page.to_string())
    }
    pub fn current_url(&self) -> String {
        self.build_url(self.current)
    }
    pub fn previous_url(&self) -> String {
        self.build_url(self.previous)
    }
    pub fn next_url(&self) -> String {
        self.build_url(self.next)
    }
    pub fn build_template_vars(&self) -> models::PaginationVars {
        models::PaginationVars {
            current: self.current,
            prev: self.previous,
            next: self.next,
            total: self.total,
            total_pages: self.total_pages,
            has_prev: self.has_previous,
            has_next: self.has_next,
            current_url: self.current_url(),
            prev_url: self.previous_url(),
            next_url: self.next_url(),
        }
    }
}

#[derive(Debug)]
pub struct Pagination {
    pub total: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

impl Pagination {
    pub fn new(total: usize, per_page: usize) -> Pagination {
        let total_pages = (total as f64 / per_page as f64).ceil() as usize;
        Pagination {
            total,
            per_page,
            total_pages,
        }
    }
    pub fn build_each_page(&self, page: usize, layout: &str) -> PaginationItem {
        PaginationItem::new(page, self.total, self.total_pages, self.per_page, layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(99, 10);
        assert_eq!(pagination.total_pages, 10);

        let page = pagination.build_each_page(3, "/page/:page");
        assert_eq!(page.current, 3);
        assert_eq!(page.total, 99);
        assert_eq!(page.total_pages, 10);
        assert_eq!(page.size, 10);
        assert_eq!(page.has_previous, true);
        assert_eq!(page.has_next, true);
        assert_eq!(page.previous, 2);
        assert_eq!(page.next, 4);
        assert_eq!(page.start, 20);
        assert_eq!(page.end, 30);
        assert_eq!(page.current_url(), "/page/3");
        assert_eq!(page.previous_url(), "/page/2");
        assert_eq!(page.next_url(), "/page/4");
    }
}
