#include <iostream>
#include <fstream>
#include <vector>
#include <map>
#include <string>
using namespace std;

vector<string> headers = {"[Event \"",
                          "[White \"",
                          "[Black \"",
                          "[WhiteElo \"",
                          "[BlackElo \"",
                          "[TimeControl \"",
                          "[Result \"",
                          "[Site \"",
                          "[Termination \"",
                          "[%eval"
                          };

string get_header(string header_name, string line) {
    std::size_t value_idx = line.find(header_name);
    if (value_idx != std::string::npos) {
        return line.substr(value_idx + header_name.size(),
                           // until end of string, except for last 2 chars
                           line.size() - value_idx - header_name.size() - 2);
    } else {
        return "";
    }
}

string get_last_eval(string line) {
    std::size_t eval_idx = line.rfind("%eval ");
    if (eval_idx != string::npos) {
        std::size_t bracket_idx = line.find("]", eval_idx + 1);
        return line.substr(eval_idx + 6, bracket_idx - eval_idx - 6);
    } else {
        return "";
    }
}

string get_last_comment(bool is_eval, string line) {
    string search_str;
    if (is_eval) {
        search_str = "%eval ";
    } else {
        search_str = "%clk ";
    }
    std::size_t eval_idx = line.rfind(search_str);
    if (eval_idx != string::npos) {
        std::size_t bracket_idx = line.find("]", eval_idx + 1);
        return line.substr(eval_idx + 6, bracket_idx);
    } else {
        return "";
    }
}

void write_to_csv(ofstream &file, map<string, string> results) {

    for (int i = 0; i < headers.size(); i++) {
        string header = headers[i];
        string ending = (i == headers.size() - 1) ? "\n" : ",";
        file << results[header] << ending;
    }
    // file << results["[White \""] << ",";
    // file << results["[Black \""] << ",";
    // file << results["[Event \""] << ",";
    // file << results["[Result \""] << ",";
    // file << results["[Termination \""] << ",";
    // file << results["[%eval"] << "\n";
    return;
}

int main() {
    string file_list[27] = {"2019-01",
                            "2019-02",
                            "2019-03",
                            "2019-04",
                            "2019-05",
                            "2019-06",
                            "2019-07",
                            "2019-08",
                            "2019-09",
                            "2019-10",
                            "2019-11",
                            "2019-12",
                            "2020-01",
                            "2020-02",
                            "2020-03",
                            "2020-04",
                            "2020-05",
                            "2020-06",
                            "2020-07",
                            "2020-08",
                            "2020-09",
                            "2020-10",
                            "2020-11",
                            "2020-12",
                            "2021-01",
                            "2021-02",
                            "2021-03",
                            };
    for (int i = 0; i < 27; i++) {
        string line;
        ifstream inputfile;
        inputfile.open("/scratch/gp1655/lichess/lichess_db_standard_rated_" + file_list[i] + ".pgn");

        if (!inputfile.is_open()) {
            cout << "Error opening file\n";
            return 1;
        }

        // string white_header = "[White \"";
        // string black_header = "[Black \"";

        map<string, string> results;
        string game_line;
        bool parsing_game = false;

        ofstream results_file;
        results_file.open("/scratch/gp1655/lichess/filtering_results_" + file_list[i] + ".csv");

        while (getline(inputfile, line)) {
            if (!parsing_game) {
                map<string, string> results;
                parsing_game = true;
            }
            for (int i = 0; i < headers.size(); i++) {
                string value = get_header(headers[i], line);
                if (value.size() > 0) {
                    results[headers[i]] = value;
                }

                if (i == (headers.size() - 1) & value.size() > 0) {
                    results[headers[i]] = get_last_eval(line);
                    parsing_game = false;
                }
            }
            if (!parsing_game) {
                write_to_csv(results_file, results);
                // for (auto item: results) {
                //     cout << "Header: " << item.first << ", value: " << item.second << "\n";
                // }
                // break;
            }
        }

        inputfile.close();
        results_file.close();
    }
    return 0;
}
