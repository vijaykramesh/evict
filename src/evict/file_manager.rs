/*
 *   Copyright 2013 Brandon Sanderson
 *
 *   This file is part of Evict-BT.
 *
 *   Evict-BT is free software: you can redistribute it and/or modify
 *   it under the terms of the GNU General Public License as published by
 *   the Free Software Foundation, either version 3 of the License, or
 *   (at your option) any later version.
 *
 *   Evict-BT is distributed in the hope that it will be useful,
 *   but WITHOUT ANY WARRANTY; without even the implied warranty of
 *   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *   GNU General Public License for more details.
 *
 *   You should have received a copy of the GNU General Public License
 *   along with Evict-BT.  If not, see <http://www.gnu.org/licenses/>.
 */
use extra;
use issue::{Issue, IssueTimelineEvent};
use file_util;
use std::option::IntoOption;
use std::io;
use extra::json::ToJson;

#[cfg(not(test))]
pub static EVICT_DIRECTORY:&'static str = ".evict";
#[cfg(test)]
pub static EVICT_DIRECTORY:&'static str = ".evict-test";

static ISSUE_DIRECTORY:&'static str = "issue-dirs";

static BODY_FILENAME:&'static str = "body";

static EXTENSION:&'static str = ".ebtd";

static LOCAL_EXT:&'static str = ".ebtdlocal";

static ACTIVE_ISSUE_FILENAME_PART:&'static str = "issues";

pub fn issue_directory() -> ~str {format!("{}/{}",
                                          EVICT_DIRECTORY,
                                          ISSUE_DIRECTORY)}

pub fn issue_directory_path() -> Path {Path::init(issue_directory())}

pub fn single_issue_filename(issue:&Issue) -> ~str {
  format!("{}/{}/{}", EVICT_DIRECTORY, ISSUE_DIRECTORY, issue.id)
}

pub fn write_issues(issues:&[Issue]) -> bool {
  write_issues_to_file(issues)
}

pub fn write_issues_to_file(issues:&[Issue]) -> bool {
  let mut allSuccess = true;
  for i in issues.iter() {
    allSuccess = allSuccess && write_single_issue(i);
  }
  allSuccess
}

fn write_single_issue(issue:&Issue) -> bool {
  file_util::create_directory(single_issue_filename(issue));
  let mut allSuccess = write_issue_body(issue);
  for event in issue.events.iter() {
    allSuccess = allSuccess && write_issue_event(issue.id, event);
  }
  allSuccess
}

fn write_issue_body(issue:&Issue) -> bool {
  let filename = issue_body_filename(issue);
  let output = issue.no_comment_json().to_pretty_str();
  file_util::write_string_to_file(output, filename, true)
}

fn issue_body_filename(issue:&Issue) -> ~str {
  format!("{}/{}/{}/{}", EVICT_DIRECTORY, ISSUE_DIRECTORY, issue.id, BODY_FILENAME)
}

fn write_issue_event(issueId:&str, event:&IssueTimelineEvent) -> bool{
  let filename = issue_event_filename(issueId, event);
  let jsonStr = event.to_json().to_pretty_str();
  file_util::write_string_to_file(jsonStr, filename, true)
}

fn issue_event_filename(issueId:&str, event:&IssueTimelineEvent) -> ~str {
  format!("{}/{}/{}/{}", EVICT_DIRECTORY, ISSUE_DIRECTORY, issueId, event.id())
}

pub fn read_issues() -> ~[Issue] {
  read_issues_from_folders()
}

fn read_issues_from_folders() -> ~[Issue] {
  /*! Reads all issues from the folders located in the
   *  folder returned by full_issue_directory.
   *  If a folder/file in the issue directory does not parse
   *  into an issue, it is ignored.
   */ 
  let dirPath = issue_directory_path();
  let issueDirs = io::fs::readdir(&dirPath);
  let issueOptions = issueDirs.move_iter().map (
    |path| read_issue_from_dir(path)
  );
  //clear all None values and unwrap Some(issue) to just issue
  issueOptions.filter_map(|x| x).collect()
}


fn read_issue_from_dir(basePath:Path) -> Option<Issue> {
  let files = io::fs::readdir(&basePath);
  let bodyPath = Path::init(BODY_FILENAME);
  let issueBodyPath = basePath.join(bodyPath);
  let noBodyFiles:~[Path] = files.move_iter()
                                 .filter(|x| x != &issueBodyPath)
                                 .collect();
  let bodyIssue = read_issue_body(issueBodyPath);
  bodyIssue.map (|mut bIssue| {
    let events = read_issue_events(noBodyFiles);
    bIssue.events = events;
    bIssue
  })
}

fn read_issue_body(bodyPath:Path) -> Option<Issue> {
  /*! Reads an issue from a file, except for the comments, which are stored
   *  separately from other data.
   */
  let dataStrOpt = file_util::read_string_from_path(&bodyPath);
  dataStrOpt.and_then(|dataStr| {
     extra::json::from_str(dataStr).into_option()
  }).and_then(|jsonVal| {
    Issue::from_json(&jsonVal)
  })
}

fn read_issue_events(bodyFiles:&[Path]) -> ~[IssueTimelineEvent] {
  bodyFiles.iter().filter_map(read_comment).collect()
}

fn read_comment(commentFile:&Path) -> Option<IssueTimelineEvent> {
  let dataStrOpt = file_util::read_string_from_path(commentFile);
  dataStrOpt.and_then(|dataStr| {
    extra::json::from_str(dataStr).into_option()
  }).and_then(|jsonVal| {
    IssueTimelineEvent::from_json(&jsonVal)
  })
}

#[test]
pub fn write_read_issue_file(){
  file_util::create_directory_path(&Path::init(EVICT_DIRECTORY));
  file_util::create_directory_path(&issue_directory_path());
  let issues = ~[Issue::new(~"A", ~"B", ~"C")];
  write_issues(issues);
  let read = read_issues();
  assert!(issues == read);
  io::fs::rmdir_recursive(&Path::init(EVICT_DIRECTORY));
}
