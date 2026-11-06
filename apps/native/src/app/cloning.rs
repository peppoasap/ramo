use super::*;
use crate::connection_work::{Job, Reply};
use crate::repository_provider::{self, Provider};
use ramo_core::{cloning, hosting};
#[derive(Default)]
pub(super) struct CloneDialog {
    provider: Provider,
    repository: String,
    host: String,
    ssh: bool,
    directory: String,
    directory_edited: bool,
    parent: Option<cloning::Grant>,
    source: Option<cloning::Grant>,
    plan: Option<cloning::Plan>,
    pub request: Option<u64>,
    pub executing: bool,
    error: Option<String>,
    organization: String,
    catalog: Vec<hosting::Repository>,
    catalog_loaded: bool,
    filter: String,
    page: u32,
}
impl CloneDialog {
    pub(super) fn receive(&mut self, result: Result<Reply, String>) -> Option<String> {
        self.request = None;
        self.executing = false;
        match result {
            Ok(Reply::Grant {
                source,
                grant: Some(grant),
            }) => {
                if source {
                    if !self.directory_edited {
                        self.directory = repository_provider::suggested(&grant.path);
                    }
                    self.source = Some(grant);
                } else {
                    self.parent = Some(grant);
                }
            }
            Ok(Reply::Grant { grant: None, .. }) => {}
            Ok(Reply::Plan(plan)) => self.plan = Some(plan),
            Ok(Reply::Cloned(path)) => return Some(path),
            Ok(Reply::Catalog(repos)) => {
                self.catalog = repos;
                self.catalog_loaded = true;
            }
            Ok(_) => self.error = Some("Unexpected clone response.".into()),
            Err(error) => self.error = Some(error),
        }
        None
    }
}
impl App {
    pub(super) fn clone_busy(&self) -> bool {
        self.clone_dialog.as_ref().is_some_and(|d| d.executing)
    }
    pub(super) fn ask_clone(&mut self) {
        if self.action_request.is_some() || self.action.is_some() || self.clone_busy() {
            return;
        }
        if self.editor.as_ref().is_some_and(|e| e.dirty) {
            self.error = Some(
                "Save the merge draft or leave the editor before cloning a repository.".into(),
            );
            return;
        }
        self.remote_dialog = None;
        self.clone_dialog = Some(CloneDialog {
            page: 1,
            ..Default::default()
        });
    }
    pub(super) fn clone_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(mut state) = self.clone_dialog.take() else {
            return;
        };
        let busy = state.request.is_some();
        let mut job = None;
        let mut close = false;
        let response = theme::modal(ctx, "clone_repository", "Clone repository", 720., |ui| {
            if let Some(error) = &state.error {
                ui.colored_label(theme::AMBER, error);
                ui.add_space(8.);
            }
            if state.executing {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.strong("Cloning repository…");
                });
                ui.label(
                    "Git is creating the new checkout. Keep this window open until it finishes.",
                );
                ui.label(theme::caption("Current clone timeout:120 seconds. A failed attempt may leave a partial folder; it is never deleted automatically."));
            } else if let Some(plan) = state.plan.clone() {
                ui.label("Source");
                ui.label(&plan.source);
                ui.add_space(8.);
                ui.label("New checkout");
                ui.label(&plan.destination);
                ui.add_space(12.);
                ui.label(if plan.network {
                    "Clone using your existing Git or SSH authentication."
                } else {
                    "Clone the selected local repository without hard links."
                });
                ui.label(theme::caption("Git transports, credential helpers, global templates and checkout filters can run local programs. The new repository opens read-only until you trust it."));
                ui.add_space(16.);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary(ui, "Clone repository", !busy).clicked() {
                        job = Some(Job::Clone(plan.id.clone()));
                    }
                    if ui.add_enabled(!busy, egui::Button::new("Back")).clicked() {
                        state.plan = None;
                    }
                });
            } else {
                ui.add_enabled_ui(!busy,|ui| {
                    ui.horizontal_top(|ui|{
                        ui.vertical(|ui|{
                            ui.set_width(155.);
                            for provider in Provider::ALL {
                                let glyph=match provider{Provider::Github|Provider::Enterprise=>icon::GITHUB_LOGO,Provider::Local=>icon::FOLDER_OPEN,Provider::Azure=>icon::CLOUD,_=>icon::GIT_BRANCH};
                                if ui.add_sized([155.,28.],egui::Button::selectable(state.provider==provider,format!("{glyph}  {}",provider.label())).frame(state.provider==provider)).clicked() {
                                    state.provider=provider;state.catalog.clear();state.catalog_loaded=false;state.error=None;
                                }
                            }
                        });
                        let (divider,_) = ui.allocate_exact_size(egui::vec2(1.,260.),Sense::hover());
                        ui.painter().line_segment([divider.center_top(),divider.center_bottom()],egui::Stroke::new(1.,theme::BORDER));
                        ui.vertical(|ui|{
                            ui.set_width(500.);
                            ScrollArea::vertical().id_salt("clone_fields").max_height((ctx.content_rect().height()-210.).clamp(260.,360.)).show(ui,|ui|{
                                ui.label(theme::emphasis(state.provider.label()));ui.add_space(8.);
                                if state.provider.custom_host() {
                                    ui.label("Server");
                                    if ui.add(TextEdit::singleline(&mut state.host).hint_text("git.example.com").desired_width(f32::INFINITY)).changed(){state.catalog.clear();state.catalog_loaded=false;}
                                }
                                if state.provider==Provider::Local {
                                    ui.label("Source repository");
                                    if let Some(source)=&state.source {ui.label(&source.path);}
                                    if theme::button(ui,icon::FOLDER_OPEN,"Choose source…").clicked(){job=Some(Job::Pick{source:true});}
                                } else {
                                    ui.label(if state.provider==Provider::Url {"Repository URL"}else if state.provider==Provider::Azure {"Organization / project / repository"}else{"Owner / repository"});
                                    if ui.add(TextEdit::singleline(&mut state.repository).hint_text(if state.provider==Provider::Url {"https://github.com/team/project.git"}else{"team/project"}).desired_width(f32::INFINITY)).changed() && !state.directory_edited {
                                        state.directory=repository_provider::suggested(&state.repository);
                                    }
                                    if state.provider!=Provider::Url {
                                        ui.horizontal(|ui|{ui.label("Connection");ui.radio_value(&mut state.ssh,false,"HTTPS");ui.radio_value(&mut state.ssh,true,"SSH");});
                                    }
                                    if !state.repository.is_empty() {match state.provider.url(&state.repository,&state.host,state.ssh){
                                        Ok(url)=>{ui.add(egui::Label::new(theme::caption(&url)).truncate()).on_hover_text(url);},
                                        Err(error)=>{ui.colored_label(theme::AMBER,error);}
                                    }}
                                }
                                if state.provider.catalog() {
                                    egui::CollapsingHeader::new("Choose from GitHub").show(ui,|ui|{
                                        ui.label(theme::caption("Load explicitly through your installed, signed-in gh CLI. No login or token import is started."));
                                        ui.horizontal(|ui|{
                                            if ui.add(TextEdit::singleline(&mut state.organization).hint_text("Organization (optional)").desired_width(270.)).changed(){state.catalog.clear();state.catalog_loaded=false;}
                                            if ui.button("Load repositories").clicked(){state.page=1;state.catalog.clear();state.catalog_loaded=false;job=Some(Job::Catalog{host:state.provider.host(&state.host).into(),organization:state.organization.trim().into(),page:1});}
                                        });
                                        if state.catalog_loaded && state.catalog.is_empty(){ui.label(theme::caption("No repositories returned for this scope. Try a different organization or paste a URL."));}
                                        if !state.catalog.is_empty(){
                                            ui.add(TextEdit::singleline(&mut state.filter).hint_text("Filter this page").desired_width(f32::INFINITY));
                                            let mut choice=None;
                                            ScrollArea::vertical().id_salt("clone_catalog").max_height(130.).show(ui,|ui|{
                                                for repo in &state.catalog {if repo.name.to_lowercase().contains(&state.filter.to_lowercase()) && ui.selectable_label(state.repository==repo.name,&repo.name).clicked(){choice=Some(repo.name.clone());}}
                                            });
                                            if let Some(name)=choice{state.repository=name;if !state.directory_edited{state.directory=repository_provider::suggested(&state.repository);}}
                                            ui.horizontal(|ui|{ui.label(theme::caption(format!("Page {} · {} repositories",state.page,state.catalog.len())));
                                                if state.catalog.len()==100 && state.page<20 && ui.small_button("Next page").clicked(){state.page+=1;state.catalog.clear();state.catalog_loaded=false;job=Some(Job::Catalog{host:state.provider.host(&state.host).into(),organization:state.organization.trim().into(),page:state.page});}
                                            });
                                        }
                                    });
                                }
                                ui.add_space(12.);ui.separator();ui.add_space(8.);
                                ui.label("Destination folder");
                                ui.horizontal_wrapped(|ui|{
                                    if let Some(parent)=&state.parent {ui.add(egui::Label::new(theme::caption(&parent.path)).truncate()).on_hover_text(&parent.path);}
                                    if theme::button(ui,icon::FOLDER_OPEN,"Choose folder…").clicked(){job=Some(Job::Pick{source:false});}
                                });
                                ui.label("New folder name");
                                if ui.add(TextEdit::singleline(&mut state.directory).desired_width(f32::INFINITY)).changed(){state.directory_edited=true;}
                                ui.label(theme::caption("Choose a parent outside existing repositories. The new folder must not already exist."));
                            });
                        });
                    });
                });
                ui.add_space(12.);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let source = if state.provider == Provider::Local {
                        state
                            .source
                            .as_ref()
                            .map(|s| cloning::Source::Local { id: s.id.clone() })
                    } else {
                        state
                            .provider
                            .url(&state.repository, &state.host, state.ssh)
                            .ok()
                            .map(|url| cloning::Source::Url { url })
                    };
                    let enabled = !busy
                        && source.is_some()
                        && state.parent.is_some()
                        && !state.directory.trim().is_empty();
                    if theme::primary(ui, "Review clone", enabled).clicked() {
                        job = Some(Job::Prepare {
                            parent: state.parent.as_ref().unwrap().id.clone(),
                            directory: state.directory.clone(),
                            source: source.unwrap(),
                        });
                    }
                    close = ui.add_enabled(!busy, egui::Button::new("Cancel")).clicked();
                    if busy {
                        ui.spinner();
                        ui.label("Loading…");
                    }
                });
            }
        });
        close |= response.should_close() && !busy;
        if let Some(job) = job {
            state.error = None;
            state.executing = matches!(job, Job::Clone(_));
            if state.executing {
                state.plan = None;
            }
            let id = self.send(Work::Connection(job));
            if self.pending.contains(&id) {
                state.request = Some(id);
            } else {
                state.executing = false;
                state.error = Some("Queue busy. Try this step again.".into());
            }
        }
        if !close || state.executing {
            self.clone_dialog = Some(state);
        }
    }
}
